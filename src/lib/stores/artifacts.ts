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

export const artifacts = writable<Artifact[]>([]);

let currentCardId: string | null = null;

export async function loadArtifacts(cardId: string): Promise<void> {
	currentCardId = cardId;
	const project = get(currentProject);
	if (!project) return;
	const list = await listArtifactsService(project.id, cardId);
	artifacts.set(list);
}

export async function addArtifact(
	cardId: string,
	name: string,
	content: string,
	createdBy: 'user' | 'agent' = 'user'
): Promise<Artifact> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const artifact = await createArtifactService(project.id, cardId, name, content, createdBy);
	await loadArtifacts(cardId);
	return artifact;
}

export async function getArtifactContent(id: string): Promise<string> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	return readArtifactService(project.id, id);
}

export async function saveArtifactContent(
	id: string,
	cardId: string,
	content: string
): Promise<Artifact> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const artifact = await updateArtifactService(project.id, id, content);
	await loadArtifacts(cardId);
	return artifact;
}

export async function removeArtifact(id: string, cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await deleteArtifactService(project.id, id);
	await loadArtifacts(cardId);
}

listenEvent<{ project_id: string }>('artifacts-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id && currentCardId) {
		loadArtifacts(currentCardId);
	}
});
