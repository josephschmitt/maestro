import { writable, get } from 'svelte/store';
import type { LinkedDirectory } from '$lib/types/index.js';
import {
	listLinkedDirectories as listLinkedDirectoriesService,
	addLinkedDirectory as addLinkedDirectoryService,
	removeLinkedDirectory as removeLinkedDirectoryService
} from '$lib/services/directories.js';
import { currentProject } from './project.js';

export const linkedDirectories = writable<LinkedDirectory[]>([]);

export async function loadLinkedDirectories(): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	const list = await listLinkedDirectoriesService(project.id);
	linkedDirectories.set(list);
}

export async function addLinkedDirectory(path: string, label: string): Promise<LinkedDirectory> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const dir = await addLinkedDirectoryService(project.id, path, label);
	await loadLinkedDirectories();
	return dir;
}

export async function removeLinkedDirectory(id: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await removeLinkedDirectoryService(project.id, id);
	await loadLinkedDirectories();
}
