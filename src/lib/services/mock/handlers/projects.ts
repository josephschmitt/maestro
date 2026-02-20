import type { Project, ProjectSummary } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

export function create_project(args: Record<string, unknown>): Project {
	const store = getStore();
	const now = nowISO();
	const project: Project = {
		id: newId(),
		name: args.name as string,
		agent_config: {},
		base_path: null,
		created_at: now,
		updated_at: now
	};
	store.projects.push(project);
	return project;
}

export function get_project(args: Record<string, unknown>): Project {
	const store = getStore();
	const project = store.projects.find((p) => p.id === args.id);
	if (!project) throw new Error(`Project not found: ${args.id}`);
	return project;
}

export function list_projects(): ProjectSummary[] {
	const store = getStore();
	return store.projects.map((p) => ({
		id: p.id,
		name: p.name,
		created_at: p.created_at
	}));
}

export function update_project(args: Record<string, unknown>): Project {
	const store = getStore();
	const project = store.projects.find((p) => p.id === args.id);
	if (!project) throw new Error(`Project not found: ${args.id}`);
	if (args.name !== undefined) project.name = args.name as string;
	if (args.agent_config !== undefined)
		project.agent_config = args.agent_config as Record<string, unknown>;
	if (args.base_path !== undefined) project.base_path = args.base_path as string | null;
	project.updated_at = nowISO();
	return project;
}

export function delete_project(args: Record<string, unknown>): void {
	const store = getStore();
	const id = args.id as string;
	store.projects = store.projects.filter((p) => p.id !== id);
	store.cards = store.cards.filter((c) => c.project_id !== id);
	store.statuses = store.statuses.filter((s) => s.project_id !== id);
}
