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
import { currentProject } from './project.js';

export const questions = writable<OpenQuestion[]>([]);

export const unresolvedQuestions = derived(questions, ($questions) =>
	$questions.filter((q) => !q.resolved_at)
);

export const resolvedQuestions = derived(questions, ($questions) =>
	$questions.filter((q) => q.resolved_at !== null)
);

export const unresolvedCountByCard = writable<Map<string, number>>(new Map());

export async function loadQuestions(cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	const list = await listQuestionsService(project.id, cardId);
	questions.set(list);
}

export async function addQuestion(
	cardId: string,
	question: string,
	source: 'agent' | 'user' = 'user'
): Promise<OpenQuestion> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const q = await createQuestionService(project.id, cardId, question, source);
	await loadQuestions(cardId);
	return q;
}

export async function resolveQuestion(
	id: string,
	cardId: string,
	resolvedBy: 'agent' | 'user' = 'user',
	resolution?: string
): Promise<OpenQuestion> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const q = await resolveQuestionService(project.id, id, resolvedBy, resolution);
	await loadQuestions(cardId);
	return q;
}

export async function unresolveQuestion(id: string, cardId: string): Promise<OpenQuestion> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const q = await unresolveQuestionService(project.id, id);
	await loadQuestions(cardId);
	return q;
}

export async function removeQuestion(id: string, cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await deleteQuestionService(project.id, id);
	await loadQuestions(cardId);
}

export async function loadUnresolvedCounts(cardIds: string[]): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	if (cardIds.length === 0) return;
	const counts = await countUnresolvedService(project.id, cardIds);
	const map = new Map<string, number>();
	for (const [cardId, count] of counts) {
		map.set(cardId, count);
	}
	unresolvedCountByCard.set(map);
}
