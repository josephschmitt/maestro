import type { Project, ProjectSummary } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function createProject(name: string): Promise<Project> {
	return tauriInvoke<Project>('create_project', { name });
}

export async function getProject(id: string): Promise<Project> {
	return tauriInvoke<Project>('get_project', { id });
}

export async function listProjects(): Promise<ProjectSummary[]> {
	return tauriInvoke<ProjectSummary[]>('list_projects');
}

export async function updateProject(
	id: string,
	updates: {
		name?: string;
		agent_config?: Record<string, unknown>;
		base_path?: string;
	}
): Promise<Project> {
	return tauriInvoke<Project>('update_project', { id, ...updates });
}

export async function deleteProject(id: string): Promise<void> {
	return tauriInvoke<void>('delete_project', { id });
}
