import type { OpenQuestion } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function createQuestion(
	projectId: string,
	cardId: string,
	question: string,
	source: 'agent' | 'user'
): Promise<OpenQuestion> {
	return tauriInvoke<OpenQuestion>('create_question', {
		projectId,
		cardId,
		question,
		source
	});
}

export async function listQuestions(
	projectId: string,
	cardId: string
): Promise<OpenQuestion[]> {
	return tauriInvoke<OpenQuestion[]>('list_questions', { projectId, cardId });
}

export async function resolveQuestion(
	projectId: string,
	id: string,
	resolvedBy: 'agent' | 'user',
	resolution?: string
): Promise<OpenQuestion> {
	return tauriInvoke<OpenQuestion>('resolve_question', {
		projectId,
		id,
		resolution: resolution ?? null,
		resolvedBy
	});
}

export async function unresolveQuestion(
	projectId: string,
	id: string
): Promise<OpenQuestion> {
	return tauriInvoke<OpenQuestion>('unresolve_question', { projectId, id });
}

export async function deleteQuestion(projectId: string, id: string): Promise<void> {
	return tauriInvoke<void>('delete_question', { projectId, id });
}

export async function countUnresolvedQuestions(
	projectId: string,
	cardIds: string[]
): Promise<[string, number][]> {
	return tauriInvoke<[string, number][]>('count_unresolved_questions', {
		projectId,
		cardIds
	});
}
