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
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const statuses = writable<Status[]>([]);
export const statusesLoading = writable(false);

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
	statusesLoading.set(true);
	try {
		const list = await listStatusesService(project.id);
		statuses.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load statuses', message);
	} finally {
		statusesLoading.set(false);
	}
}

export async function addStatus(group: StatusGroup, name: string, isDefault?: boolean): Promise<Status | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const status = await createStatusService(project.id, group, name, isDefault);
		await loadStatuses();
		toasts.success('Status created', `"${name}" has been added.`);
		return status;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to create status', message);
		return null;
	}
}

export async function updateStatus(
	id: string,
	updates: { name?: string; isDefault?: boolean; statusPrompts?: string[] }
): Promise<Status | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const status = await updateStatusService(project.id, id, updates);
		await loadStatuses();
		return status;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to update status', message);
		return null;
	}
}

export async function removeStatus(id: string): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await deleteStatusService(project.id, id);
		await loadStatuses();
		toasts.success('Status deleted', 'The status has been removed.');
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to delete status', message);
		return false;
	}
}

export async function reorderStatuses(group: StatusGroup, statusIds: string[]): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await reorderStatusesService(project.id, group, statusIds);
		await loadStatuses();
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to reorder statuses', message);
		return false;
	}
}

listenEvent<{ project_id: string }>('statuses-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id) {
		loadStatuses();
	}
});
