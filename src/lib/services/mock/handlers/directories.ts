import type { LinkedDirectory } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

export function add_linked_directory(args: Record<string, unknown>): LinkedDirectory {
	const store = getStore();
	const path = args.path as string;
	const label = args.label as string;
	const projectId = args.projectId as string;

	const existing = store.linkedDirectories.find(
		(d) => d.project_id === projectId && d.path === path
	);
	if (existing) {
		throw new Error(`Directory already linked: ${path}`);
	}

	const dir: LinkedDirectory = {
		id: newId(),
		project_id: projectId,
		path,
		label,
		is_repo: path.includes('repo') || path.includes('git'),
		created_at: nowISO()
	};
	store.linkedDirectories.push(dir);
	return dir;
}

export function remove_linked_directory(args: Record<string, unknown>): void {
	const store = getStore();
	const id = args.id as string;
	const before = store.linkedDirectories.length;
	store.linkedDirectories = store.linkedDirectories.filter((d) => d.id !== id);
	if (store.linkedDirectories.length === before) {
		throw new Error(`Linked directory ${id} not found`);
	}
}

export function list_linked_directories(args: Record<string, unknown>): LinkedDirectory[] {
	const store = getStore();
	return store.linkedDirectories
		.filter((d) => d.project_id === args.projectId)
		.sort((a, b) => a.created_at.localeCompare(b.created_at));
}
