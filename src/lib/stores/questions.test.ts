import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { OpenQuestion } from '$lib/types/index.js';

vi.mock('$lib/services/questions.js', () => ({
	listQuestions: vi.fn(),
	createQuestion: vi.fn(),
	resolveQuestion: vi.fn(),
	unresolveQuestion: vi.fn(),
	deleteQuestion: vi.fn(),
	countUnresolvedQuestions: vi.fn()
}));

vi.mock('./project.js', async () => {
	const { writable } = await import('svelte/store');
	return {
		currentProject: writable({ id: 'project-1', name: 'Test' })
	};
});

import {
	questions,
	unresolvedQuestions,
	resolvedQuestions,
	unresolvedCountByCard,
	loadQuestions,
	addQuestion,
	resolveQuestion,
	unresolveQuestion,
	removeQuestion,
	loadUnresolvedCounts
} from './questions.js';
import * as questionsService from '$lib/services/questions.js';

function makeQuestion(overrides: Partial<OpenQuestion> = {}): OpenQuestion {
	return {
		id: 'q-1',
		card_id: 'card-1',
		question: 'Test question?',
		resolution: null,
		source: 'user',
		resolved_by: null,
		created_at: '2026-01-01T00:00:00Z',
		resolved_at: null,
		...overrides
	};
}

beforeEach(() => {
	vi.clearAllMocks();
	questions.set([]);
	unresolvedCountByCard.set(new Map());
});

describe('questions store', () => {
	describe('derived stores', () => {
		it('splits questions into resolved and unresolved', () => {
			const q1 = makeQuestion({ id: 'q-1', resolved_at: null });
			const q2 = makeQuestion({ id: 'q-2', resolved_at: '2026-01-02T00:00:00Z', resolution: 'Done' });
			questions.set([q1, q2]);

			expect(get(unresolvedQuestions)).toEqual([q1]);
			expect(get(resolvedQuestions)).toEqual([q2]);
		});
	});

	describe('loadQuestions', () => {
		it('fetches questions for a card', async () => {
			const q = makeQuestion();
			vi.mocked(questionsService.listQuestions).mockResolvedValue([q]);

			await loadQuestions('card-1');

			expect(questionsService.listQuestions).toHaveBeenCalledWith('project-1', 'card-1');
			expect(get(questions)).toEqual([q]);
		});
	});

	describe('addQuestion', () => {
		it('creates a question and reloads', async () => {
			const q = makeQuestion();
			vi.mocked(questionsService.createQuestion).mockResolvedValue(q);
			vi.mocked(questionsService.listQuestions).mockResolvedValue([q]);

			const result = await addQuestion('card-1', 'Test question?');

			expect(questionsService.createQuestion).toHaveBeenCalledWith('project-1', 'card-1', 'Test question?', 'user');
			expect(result).toEqual(q);
		});
	});

	describe('resolveQuestion', () => {
		it('resolves a question and reloads', async () => {
			const resolved = makeQuestion({ resolved_at: '2026-01-02T00:00:00Z', resolution: 'Answer' });
			vi.mocked(questionsService.resolveQuestion).mockResolvedValue(resolved);
			vi.mocked(questionsService.listQuestions).mockResolvedValue([resolved]);

			await resolveQuestion('q-1', 'card-1', 'user', 'Answer');

			expect(questionsService.resolveQuestion).toHaveBeenCalledWith('project-1', 'q-1', 'user', 'Answer');
		});
	});

	describe('unresolveQuestion', () => {
		it('unresolves a question and reloads', async () => {
			const q = makeQuestion();
			vi.mocked(questionsService.unresolveQuestion).mockResolvedValue(q);
			vi.mocked(questionsService.listQuestions).mockResolvedValue([q]);

			await unresolveQuestion('q-1', 'card-1');

			expect(questionsService.unresolveQuestion).toHaveBeenCalledWith('project-1', 'q-1');
		});
	});

	describe('removeQuestion', () => {
		it('deletes a question and reloads', async () => {
			vi.mocked(questionsService.deleteQuestion).mockResolvedValue();
			vi.mocked(questionsService.listQuestions).mockResolvedValue([]);

			await removeQuestion('q-1', 'card-1');

			expect(questionsService.deleteQuestion).toHaveBeenCalledWith('project-1', 'q-1');
			expect(get(questions)).toEqual([]);
		});
	});

	describe('loadUnresolvedCounts', () => {
		it('loads counts for multiple cards', async () => {
			vi.mocked(questionsService.countUnresolvedQuestions).mockResolvedValue([
				['card-1', 3],
				['card-2', 0],
				['card-3', 1]
			]);

			await loadUnresolvedCounts(['card-1', 'card-2', 'card-3']);

			const counts = get(unresolvedCountByCard);
			expect(counts.get('card-1')).toBe(3);
			expect(counts.get('card-2')).toBe(0);
			expect(counts.get('card-3')).toBe(1);
		});

		it('skips loading for empty card list', async () => {
			await loadUnresolvedCounts([]);
			expect(questionsService.countUnresolvedQuestions).not.toHaveBeenCalled();
		});
	});
});
