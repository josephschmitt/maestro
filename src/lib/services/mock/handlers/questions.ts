import type { OpenQuestion } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

export function create_question(args: Record<string, unknown>): OpenQuestion {
	const store = getStore();
	const question: OpenQuestion = {
		id: newId(),
		card_id: args.cardId as string,
		question: args.question as string,
		resolution: null,
		source: args.source as 'agent' | 'user',
		resolved_by: null,
		created_at: nowISO(),
		resolved_at: null
	};
	store.questions.push(question);
	return question;
}

export function list_questions(args: Record<string, unknown>): OpenQuestion[] {
	const store = getStore();
	return store.questions
		.filter((q) => q.card_id === args.cardId)
		.sort((a, b) => a.created_at.localeCompare(b.created_at));
}

export function resolve_question(args: Record<string, unknown>): OpenQuestion {
	const store = getStore();
	const question = store.questions.find((q) => q.id === args.id);
	if (!question) throw new Error(`Question not found: ${args.id}`);
	question.resolution = (args.resolution as string) ?? null;
	question.resolved_by = args.resolvedBy as 'agent' | 'user';
	question.resolved_at = nowISO();
	return question;
}

export function unresolve_question(args: Record<string, unknown>): OpenQuestion {
	const store = getStore();
	const question = store.questions.find((q) => q.id === args.id);
	if (!question) throw new Error(`Question not found: ${args.id}`);
	question.resolution = null;
	question.resolved_by = null;
	question.resolved_at = null;
	return question;
}

export function delete_question(args: Record<string, unknown>): void {
	const store = getStore();
	store.questions = store.questions.filter((q) => q.id !== args.id);
}

export function count_unresolved_questions(args: Record<string, unknown>): [string, number][] {
	const store = getStore();
	const cardIds = args.cardIds as string[];
	return cardIds.map((cardId) => {
		const count = store.questions.filter(
			(q) => q.card_id === cardId && q.resolved_at === null
		).length;
		return [cardId, count];
	});
}
