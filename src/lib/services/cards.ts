import type { CardWithStatus } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function createCard(
	projectId: string,
	title: string,
	options?: {
		description?: string;
		labels?: string[];
		parentId?: string;
		statusId?: string;
	}
): Promise<CardWithStatus> {
	return tauriInvoke<CardWithStatus>('create_card', {
		projectId,
		title,
		description: options?.description,
		labels: options?.labels,
		parentId: options?.parentId,
		statusId: options?.statusId
	});
}

export async function getCard(projectId: string, id: string): Promise<CardWithStatus> {
	return tauriInvoke<CardWithStatus>('get_card', { projectId, id });
}

export async function updateCard(
	projectId: string,
	id: string,
	updates: { title?: string; description?: string; labels?: string[] }
): Promise<CardWithStatus> {
	return tauriInvoke<CardWithStatus>('update_card', { projectId, id, ...updates });
}

export async function deleteCard(projectId: string, id: string): Promise<void> {
	return tauriInvoke<void>('delete_card', { projectId, id });
}

export async function listCards(projectId: string): Promise<CardWithStatus[]> {
	return tauriInvoke<CardWithStatus[]>('list_cards', { projectId });
}

export async function listSubCards(
	projectId: string,
	parentId: string
): Promise<CardWithStatus[]> {
	return tauriInvoke<CardWithStatus[]>('list_sub_cards', { projectId, parentId });
}

export async function moveCard(
	projectId: string,
	id: string,
	targetStatusId: string,
	targetSortOrder: number
): Promise<CardWithStatus> {
	return tauriInvoke<CardWithStatus>('move_card', {
		projectId,
		id,
		targetStatusId,
		targetSortOrder
	});
}

export async function reorderCards(
	projectId: string,
	statusId: string,
	cardIds: string[]
): Promise<CardWithStatus[]> {
	return tauriInvoke<CardWithStatus[]>('reorder_cards', { projectId, statusId, cardIds });
}
