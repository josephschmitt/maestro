import type { DndEvent } from 'svelte-dnd-action';
import type { CardWithStatus } from '$lib/types/index.js';
import { moveCard, reorderCards } from '$lib/stores/cards.js';

export const FLIP_DURATION_MS = 200;

export const DND_TYPE = 'kanban-card';

export type DndItem = CardWithStatus & { isDndShadowItem?: boolean };

/**
 * Handle the finalize event from svelte-dnd-action.
 * Determines whether this is a same-column reorder or cross-column move
 * and calls the appropriate store function to persist to DB.
 */
export async function handleFinalize(
	event: CustomEvent<DndEvent<DndItem>>,
	statusId: string,
	originalStatusId: (cardId: string) => string | undefined
): Promise<void> {
	const { items, info } = event.detail;
	const draggedId = info.id;

	const sourceStatusId = originalStatusId(draggedId);
	if (!sourceStatusId) return;

	const newIndex = items.findIndex((item) => item.id === draggedId);
	if (newIndex === -1) return;

	if (sourceStatusId === statusId) {
		// Same-column reorder: send the full ordered list of card IDs
		const cardIds = items.filter((item) => !item.isDndShadowItem).map((item) => item.id);
		await reorderCards(statusId, cardIds);
	} else {
		// Cross-column move: move to new status at the target index
		await moveCard(draggedId, statusId, newIndex);
	}
}
