import { writable, get } from 'svelte/store';
import type { Artifact } from '$lib/types/index.js';
import {
	listArtifacts as listArtifactsService,
	createArtifact as createArtifactService,
	readArtifact as readArtifactService,
	updateArtifact as updateArtifactService,
	deleteArtifact as deleteArtifactService
} from '$lib/services/artifacts.js';
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const artifacts = writable<Artifact[]>([]);
export const artifactsLoading = writable(false);

let currentCardId: string | null = null;

export async function loadArtifacts(cardId: string): Promise<void> {
	currentCardId = cardId;
	const project = get(currentProject);
	if (!project) return;
	artifactsLoading.set(true);
	try {
		const list = await listArtifactsService(project.id, cardId);
		artifacts.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load artifacts', message);
	} finally {
		artifactsLoading.set(false);
	}
}

export async function addArtifact(
	cardId: string,
	name: string,
	content: string,
	createdBy: 'user' | 'agent' = 'user'
): Promise<Artifact | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const artifact = await createArtifactService(project.id, cardId, name, content, createdBy);
		await loadArtifacts(cardId);
		toasts.success('Artifact created', `"${name}" has been added.`);
		return artifact;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to create artifact', message);
		return null;
	}
}

export async function getArtifactContent(id: string): Promise<string | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		return await readArtifactService(project.id, id);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to read artifact', message);
		return null;
	}
}

export async function saveArtifactContent(
	id: string,
	cardId: string,
	content: string
): Promise<Artifact | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const artifact = await updateArtifactService(project.id, id, content);
		await loadArtifacts(cardId);
		toasts.success('Artifact saved', 'Your changes have been saved.');
		return artifact;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to save artifact', message);
		return null;
	}
}

export async function removeArtifact(id: string, cardId: string): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await deleteArtifactService(project.id, id);
		await loadArtifacts(cardId);
		toasts.success('Artifact deleted', 'The artifact has been removed.');
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to delete artifact', message);
		return false;
	}
}

listenEvent<{ project_id: string }>('artifacts-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id && currentCardId) {
		loadArtifacts(currentCardId);
	}
});
