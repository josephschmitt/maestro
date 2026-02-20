import type { CardWithStatus } from '$lib/types/index.js';
import { getStore, newId, nowISO, enrichCard } from '../store.js';

export function create_card(args: Record<string, unknown>): CardWithStatus {
	const store = getStore();
	const projectId = args.projectId as string;

	let statusId = args.statusId as string | undefined;
	if (!statusId) {
		const defaultStatus = store.statuses.find(
			(s) => s.project_id === projectId && s.is_default && s.group === 'Unstarted'
		);
		statusId = defaultStatus?.id ?? store.statuses.find((s) => s.project_id === projectId)?.id;
	}
	if (!statusId) throw new Error('No statuses found for project');

	const siblings = store.cards.filter(
		(c) => c.status_id === statusId && c.project_id === projectId
	);
	const maxOrder = siblings.reduce((max, c) => Math.max(max, c.sort_order), -1);

	const now = nowISO();
	const card = {
		id: newId(),
		project_id: projectId,
		parent_id: (args.parentId as string) ?? null,
		status_id: statusId,
		title: args.title as string,
		description: (args.description as string) ?? '',
		labels: (args.labels as string[]) ?? [],
		sort_order: maxOrder + 1,
		created_at: now,
		updated_at: now
	};
	store.cards.push(card);
	return enrichCard(card, store.statuses);
}

export function get_card(args: Record<string, unknown>): CardWithStatus {
	const store = getStore();
	const card = store.cards.find(
		(c) => c.id === args.id && c.project_id === args.projectId
	);
	if (!card) throw new Error(`Card not found: ${args.id}`);
	return enrichCard(card, store.statuses);
}

export function update_card(args: Record<string, unknown>): CardWithStatus {
	const store = getStore();
	const card = store.cards.find(
		(c) => c.id === args.id && c.project_id === args.projectId
	);
	if (!card) throw new Error(`Card not found: ${args.id}`);
	if (args.title !== undefined) card.title = args.title as string;
	if (args.description !== undefined) card.description = args.description as string;
	if (args.labels !== undefined) card.labels = args.labels as string[];
	card.updated_at = nowISO();
	return enrichCard(card, store.statuses);
}

export function delete_card(args: Record<string, unknown>): void {
	const store = getStore();
	store.cards = store.cards.filter((c) => c.id !== args.id);
}

export function list_cards(args: Record<string, unknown>): CardWithStatus[] {
	const store = getStore();
	return store.cards
		.filter((c) => c.project_id === args.projectId && c.parent_id === null)
		.sort((a, b) => a.sort_order - b.sort_order)
		.map((c) => enrichCard(c, store.statuses));
}

export function list_sub_cards(args: Record<string, unknown>): CardWithStatus[] {
	const store = getStore();
	return store.cards
		.filter((c) => c.project_id === args.projectId && c.parent_id === args.parentId)
		.sort((a, b) => a.sort_order - b.sort_order)
		.map((c) => enrichCard(c, store.statuses));
}

export function move_card(args: Record<string, unknown>): CardWithStatus {
	const store = getStore();
	const card = store.cards.find(
		(c) => c.id === args.id && c.project_id === args.projectId
	);
	if (!card) throw new Error(`Card not found: ${args.id}`);
	card.status_id = args.targetStatusId as string;
	card.sort_order = args.targetSortOrder as number;
	card.updated_at = nowISO();
	return enrichCard(card, store.statuses);
}

export function reorder_cards(args: Record<string, unknown>): CardWithStatus[] {
	const store = getStore();
	const cardIds = args.cardIds as string[];
	cardIds.forEach((id, index) => {
		const card = store.cards.find((c) => c.id === id);
		if (card) card.sort_order = index;
	});
	const statusId = args.statusId as string;
	return store.cards
		.filter((c) => c.status_id === statusId && c.project_id === args.projectId)
		.sort((a, b) => a.sort_order - b.sort_order)
		.map((c) => enrichCard(c, store.statuses));
}
