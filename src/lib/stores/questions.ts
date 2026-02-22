import { writable, derived, get } from 'svelte/store';
import type { OpenQuestion } from '$lib/types/index.js';
import {
	listQuestions as listQuestionsService,
	createQuestion as createQuestionService,
	resolveQuestion as resolveQuestionService,
	unresolveQuestion as unresolveQuestionService,
	deleteQuestion as deleteQuestionService,
	countUnresolvedQuestions as countUnresolvedService
} from '$lib/services/questions.js';
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const questions = writable<OpenQuestion[]>([]);
export const questionsLoading = writable(false);

export const unresolvedQuestions = derived(questions, ($questions) =>
	$questions.filter((q) => !q.resolved_at)
);

export const resolvedQuestions = derived(questions, ($questions) =>
	$questions.filter((q) => q.resolved_at !== null)
);

export const unresolvedCountByCard = writable<Map<string, number>>(new Map());

let currentCardId: string | null = null;

export async function loadQuestions(cardId: string): Promise<void> {
	currentCardId = cardId;
	const project = get(currentProject);
	if (!project) return;
	questionsLoading.set(true);
	try {
		const list = await listQuestionsService(project.id, cardId);
		questions.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load questions', message);
	} finally {
		questionsLoading.set(false);
	}
}

export async function addQuestion(
	cardId: string,
	question: string,
	source: 'agent' | 'user' = 'user'
): Promise<OpenQuestion | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const q = await createQuestionService(project.id, cardId, question, source);
		await loadQuestions(cardId);
		return q;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to add question', message);
		return null;
	}
}

export async function resolveQuestion(
	id: string,
	cardId: string,
	resolvedBy: 'agent' | 'user' = 'user',
	resolution?: string
): Promise<OpenQuestion | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const q = await resolveQuestionService(project.id, id, resolvedBy, resolution);
		await loadQuestions(cardId);
		return q;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to resolve question', message);
		return null;
	}
}

export async function unresolveQuestion(id: string, cardId: string): Promise<OpenQuestion | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const q = await unresolveQuestionService(project.id, id);
		await loadQuestions(cardId);
		return q;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to unresolve question', message);
		return null;
	}
}

export async function removeQuestion(id: string, cardId: string): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await deleteQuestionService(project.id, id);
		await loadQuestions(cardId);
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to delete question', message);
		return false;
	}
}

export async function loadUnresolvedCounts(cardIds: string[]): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	if (cardIds.length === 0) return;
	try {
		const counts = await countUnresolvedService(project.id, cardIds);
		const map = new Map<string, number>();
		for (const [cardId, count] of counts) {
			map.set(cardId, count);
		}
		unresolvedCountByCard.set(map);
	} catch (error) {
		console.error('Failed to load unresolved counts:', error);
	}
}

listenEvent<{ project_id: string }>('questions-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id && currentCardId) {
		loadQuestions(currentCardId);
	}
});
