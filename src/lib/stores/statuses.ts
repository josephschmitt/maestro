import { writable, derived, get } from 'svelte/store';
import type { Status, StatusGroup } from '$lib/types/index.js';
import { STATUS_GROUPS } from '$lib/types/index.js';
import {
	listStatuses as listStatusesService,
	createStatus as createStatusService,
	updateStatus as updateStatusService,
	deleteStatus as deleteStatusService,
	reorderStatuses as reorderStatusesService
} from '$lib/services/statuses.js';
import { currentProject } from './project.js';

export const statuses = writable<Status[]>([]);

export const statusesByGroup = derived(statuses, ($statuses) => {
	const map = new Map<StatusGroup, Status[]>();
	for (const group of STATUS_GROUPS) {
		map.set(
			group,
			$statuses.filter((s) => s.group === group).sort((a, b) => a.sort_order - b.sort_order)
		);
	}
	return map;
});

export async function loadStatuses(): Promise<void> {
	const project = get(currentProject);
	if (!project) {
		statuses.set([]);
		return;
	}
	const list = await listStatusesService(project.id);
	statuses.set(list);
}

export async function addStatus(group: StatusGroup, name: string, isDefault?: boolean): Promise<Status> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const status = await createStatusService(project.id, group, name, isDefault);
	await loadStatuses();
	return status;
}

export async function updateStatus(
	id: string,
	updates: { name?: string; isDefault?: boolean }
): Promise<Status> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const status = await updateStatusService(project.id, id, updates);
	await loadStatuses();
	return status;
}

export async function removeStatus(id: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await deleteStatusService(project.id, id);
	await loadStatuses();
}

export async function reorderStatuses(group: StatusGroup, statusIds: string[]): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await reorderStatusesService(project.id, group, statusIds);
	await loadStatuses();
}
