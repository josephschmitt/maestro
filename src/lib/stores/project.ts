import { writable, derived } from 'svelte/store';
import type { Project, ProjectSummary } from '$lib/types/index.js';
import { listProjects, getProject, createProject as createProjectService } from '$lib/services/projects.js';
import { getGlobalConfig, setLastProject } from '$lib/services/config.js';
import { listenEvent } from '$lib/services/events.js';

export const projects = writable<ProjectSummary[]>([]);
export const currentProject = writable<Project | null>(null);
export const isLoading = writable(false);

export const hasProject = derived(currentProject, ($project) => $project !== null);

export async function loadProjects(): Promise<void> {
	const list = await listProjects();
	projects.set(list);
}

export async function switchProject(id: string): Promise<void> {
	isLoading.set(true);
	try {
		const project = await getProject(id);
		currentProject.set(project);
		await setLastProject(id);
	} finally {
		isLoading.set(false);
	}
}

export async function createProject(name: string): Promise<Project> {
	const project = await createProjectService(name);
	await loadProjects();
	currentProject.set(project);
	return project;
}

export async function reloadCurrentProject(): Promise<void> {
	let id: string | null = null;
	const unsubscribe = currentProject.subscribe((p) => {
		id = p?.id ?? null;
	});
	unsubscribe();
	if (id) {
		const project = await getProject(id);
		currentProject.set(project);
	}
}

export async function initializeProject(): Promise<void> {
	isLoading.set(true);
	try {
		await loadProjects();
		const config = await getGlobalConfig();
		if (config.last_project_id) {
			try {
				await switchProject(config.last_project_id);
			} catch {
				currentProject.set(null);
			}
		}
	} catch {
		projects.set([]);
		currentProject.set(null);
	} finally {
		isLoading.set(false);
	}
}

listenEvent('projects-changed', () => {
	loadProjects();
});
