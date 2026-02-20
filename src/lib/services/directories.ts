import type { LinkedDirectory } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function addLinkedDirectory(
	projectId: string,
	path: string,
	label: string
): Promise<LinkedDirectory> {
	return tauriInvoke<LinkedDirectory>('add_linked_directory', {
		projectId,
		path,
		label
	});
}

export async function removeLinkedDirectory(projectId: string, id: string): Promise<void> {
	return tauriInvoke<void>('remove_linked_directory', { projectId, id });
}

export async function listLinkedDirectories(projectId: string): Promise<LinkedDirectory[]> {
	return tauriInvoke<LinkedDirectory[]>('list_linked_directories', { projectId });
}
