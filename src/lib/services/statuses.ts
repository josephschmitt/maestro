import type { Status } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function listStatuses(projectId: string): Promise<Status[]> {
	return tauriInvoke<Status[]>('list_statuses', { projectId });
}

export async function createStatus(
	projectId: string,
	group: string,
	name: string,
	isDefault?: boolean
): Promise<Status> {
	return tauriInvoke<Status>('create_status', { projectId, group, name, isDefault });
}

export async function updateStatus(
	projectId: string,
	id: string,
	updates: { name?: string; isDefault?: boolean }
): Promise<Status> {
	return tauriInvoke<Status>('update_status', { projectId, id, ...updates });
}

export async function deleteStatus(projectId: string, id: string): Promise<void> {
	return tauriInvoke<void>('delete_status', { projectId, id });
}

export async function reorderStatuses(
	projectId: string,
	group: string,
	statusIds: string[]
): Promise<Status[]> {
	return tauriInvoke<Status[]>('reorder_statuses', { projectId, group, statusIds });
}
