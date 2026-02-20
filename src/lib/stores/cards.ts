import { writable, derived, get } from 'svelte/store';
import type { CardWithStatus, CardProgress } from '$lib/types/index.js';
import {
	listCards as listCardsService,
	listSubCards as listSubCardsService,
	createCard as createCardService,
	updateCard as updateCardService,
	deleteCard as deleteCardService,
	moveCard as moveCardService,
	reorderCards as reorderCardsService
} from '$lib/services/cards.js';
import { currentProject } from './project.js';
import { statuses } from './statuses.js';
import { linkedDirectories } from './directories.js';

export const cards = writable<CardWithStatus[]>([]);

export const showLinkDirectoryPrompt = writable(false);

export const cardsByStatus = derived(cards, ($cards) => {
	const map = new Map<string, CardWithStatus[]>();
	for (const card of $cards) {
		const list = map.get(card.status_id) ?? [];
		list.push(card);
		map.set(card.status_id, list);
	}
	for (const [key, list] of map) {
		map.set(
			key,
			list.sort((a, b) => a.sort_order - b.sort_order)
		);
	}
	return map;
});

export async function loadCards(): Promise<void> {
	const project = get(currentProject);
	if (!project) {
		cards.set([]);
		return;
	}
	const list = await listCardsService(project.id);
	cards.set(list);
}

export async function addCard(
	title: string,
	options?: {
		description?: string;
		labels?: string[];
		parentId?: string;
		statusId?: string;
	}
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const card = await createCardService(project.id, title, options);
	await loadCards();
	return card;
}

export async function updateCard(
	id: string,
	updates: { title?: string; description?: string; labels?: string[] }
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const card = await updateCardService(project.id, id, updates);
	await loadCards();
	return card;
}

export async function removeCard(id: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await deleteCardService(project.id, id);
	await loadCards();
}

export async function moveCard(
	id: string,
	targetStatusId: string,
	targetSortOrder: number
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	const targetStatus = get(statuses).find((s) => s.id === targetStatusId);
	if (targetStatus?.group === 'Unstarted') {
		const dirs = get(linkedDirectories);
		if (dirs.length === 0) {
			showLinkDirectoryPrompt.set(true);
		}
	}

	const card = await moveCardService(project.id, id, targetStatusId, targetSortOrder);
	await loadCards();
	return card;
}

export async function reorderCards(
	statusId: string,
	cardIds: string[]
): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await reorderCardsService(project.id, statusId, cardIds);
	await loadCards();
}

export async function getSubCards(parentId: string): Promise<CardWithStatus[]> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	return listSubCardsService(project.id, parentId);
}

export function getCardProgress(parentId: string): CardProgress {
	const $cards = get(cards);
	const subCards = $cards.filter((c) => c.parent_id === parentId);
	const completed = subCards.filter((c) => c.status_group === 'Completed').length;
	return { completed, total: subCards.length };
}
