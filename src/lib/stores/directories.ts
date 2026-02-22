import { writable, get } from 'svelte/store';
import type { LinkedDirectory } from '$lib/types/index.js';
import {
	listLinkedDirectories as listLinkedDirectoriesService,
	addLinkedDirectory as addLinkedDirectoryService,
	removeLinkedDirectory as removeLinkedDirectoryService
} from '$lib/services/directories.js';
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const linkedDirectories = writable<LinkedDirectory[]>([]);
export const directoriesLoading = writable(false);

export async function loadLinkedDirectories(): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	directoriesLoading.set(true);
	try {
		const list = await listLinkedDirectoriesService(project.id);
		linkedDirectories.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load directories', message);
	} finally {
		directoriesLoading.set(false);
	}
}

export async function addLinkedDirectory(path: string, label: string): Promise<LinkedDirectory | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const dir = await addLinkedDirectoryService(project.id, path, label);
		await loadLinkedDirectories();
		toasts.success('Directory linked', `"${label}" has been linked.`);
		return dir;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to link directory', message);
		return null;
	}
}

export async function removeLinkedDirectory(id: string): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await removeLinkedDirectoryService(project.id, id);
		await loadLinkedDirectories();
		toasts.success('Directory unlinked', 'The directory has been removed.');
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to unlink directory', message);
		return false;
	}
}

listenEvent<{ project_id: string }>('directories-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id) {
		loadLinkedDirectories();
	}
});
