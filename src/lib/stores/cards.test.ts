import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { CardWithStatus } from '$lib/types/index.js';

vi.mock('$lib/services/cards.js', () => ({
	listCards: vi.fn(),
	listSubCards: vi.fn(),
	createCard: vi.fn(),
	updateCard: vi.fn(),
	deleteCard: vi.fn(),
	moveCard: vi.fn(),
	reorderCards: vi.fn()
}));

vi.mock('./project.js', async () => {
	const { writable } = await import('svelte/store');
	return {
		currentProject: writable({ id: 'project-1', name: 'Test' })
	};
});

import { cards, cardsByStatus, loadCards, addCard, moveCard, removeCard, getCardProgress } from './cards.js';
import * as cardsService from '$lib/services/cards.js';

function makeCard(overrides: Partial<CardWithStatus> = {}): CardWithStatus {
	return {
		id: 'card-1',
		project_id: 'project-1',
		parent_id: null,
		status_id: 'status-1',
		title: 'Test Card',
		description: '',
		labels: [],
		sort_order: 0,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		status_name: 'Backlog',
		status_group: 'Backlog',
		...overrides
	};
}

describe('cards store', () => {
	beforeEach(() => {
		cards.set([]);
		vi.clearAllMocks();
	});

	describe('cardsByStatus', () => {
		it('groups cards by status_id', () => {
			const cardA = makeCard({ id: 'a', status_id: 's1', sort_order: 0 });
			const cardB = makeCard({ id: 'b', status_id: 's1', sort_order: 1 });
			const cardC = makeCard({ id: 'c', status_id: 's2', sort_order: 0 });

			cards.set([cardA, cardB, cardC]);

			const map = get(cardsByStatus);
			expect(map.get('s1')).toHaveLength(2);
			expect(map.get('s2')).toHaveLength(1);
		});

		it('sorts cards within status by sort_order', () => {
			const cardA = makeCard({ id: 'a', status_id: 's1', sort_order: 2 });
			const cardB = makeCard({ id: 'b', status_id: 's1', sort_order: 0 });
			const cardC = makeCard({ id: 'c', status_id: 's1', sort_order: 1 });

			cards.set([cardA, cardB, cardC]);

			const list = get(cardsByStatus).get('s1')!;
			expect(list[0].id).toBe('b');
			expect(list[1].id).toBe('c');
			expect(list[2].id).toBe('a');
		});

		it('returns empty map when no cards', () => {
			cards.set([]);
			const map = get(cardsByStatus);
			expect(map.size).toBe(0);
		});
	});

	describe('loadCards', () => {
		it('fetches cards from service and updates store', async () => {
			const mockCards = [makeCard({ id: '1' }), makeCard({ id: '2' })];
			vi.mocked(cardsService.listCards).mockResolvedValue(mockCards);

			await loadCards();

			expect(cardsService.listCards).toHaveBeenCalledWith('project-1');
			expect(get(cards)).toEqual(mockCards);
		});
	});

	describe('addCard', () => {
		it('creates card via service and reloads', async () => {
			const newCard = makeCard({ id: 'new' });
			vi.mocked(cardsService.createCard).mockResolvedValue(newCard);
			vi.mocked(cardsService.listCards).mockResolvedValue([newCard]);

			const result = await addCard('New Card', { description: 'desc' });

			expect(cardsService.createCard).toHaveBeenCalledWith('project-1', 'New Card', {
				description: 'desc'
			});
			expect(result.id).toBe('new');
		});
	});

	describe('moveCard', () => {
		it('moves card via service and reloads', async () => {
			const moved = makeCard({ id: 'a', status_id: 's2' });
			vi.mocked(cardsService.moveCard).mockResolvedValue(moved);
			vi.mocked(cardsService.listCards).mockResolvedValue([moved]);

			const result = await moveCard('a', 's2', 0);

			expect(cardsService.moveCard).toHaveBeenCalledWith('project-1', 'a', 's2', 0);
			expect(result.status_id).toBe('s2');
		});
	});

	describe('removeCard', () => {
		it('deletes card via service and reloads', async () => {
			vi.mocked(cardsService.deleteCard).mockResolvedValue();
			vi.mocked(cardsService.listCards).mockResolvedValue([]);

			await removeCard('a');

			expect(cardsService.deleteCard).toHaveBeenCalledWith('project-1', 'a');
			expect(get(cards)).toEqual([]);
		});
	});

	describe('getCardProgress', () => {
		it('counts completed sub-cards', () => {
			cards.set([
				makeCard({ id: 'parent', parent_id: null }),
				makeCard({ id: 'child-1', parent_id: 'parent', status_group: 'Started' }),
				makeCard({ id: 'child-2', parent_id: 'parent', status_group: 'Completed' }),
				makeCard({ id: 'child-3', parent_id: 'parent', status_group: 'Completed' })
			]);

			const progress = getCardProgress('parent');
			expect(progress.completed).toBe(2);
			expect(progress.total).toBe(3);
		});

		it('returns zero for card with no sub-cards', () => {
			cards.set([makeCard({ id: 'solo', parent_id: null })]);
			const progress = getCardProgress('solo');
			expect(progress.completed).toBe(0);
			expect(progress.total).toBe(0);
		});
	});
});
