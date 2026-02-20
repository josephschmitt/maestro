import type { Artifact } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function createArtifact(
	projectId: string,
	cardId: string,
	name: string,
	content: string,
	createdBy: 'user' | 'agent'
): Promise<Artifact> {
	return tauriInvoke<Artifact>('create_artifact', {
		projectId,
		cardId,
		name,
		content,
		createdBy
	});
}

export async function readArtifact(projectId: string, id: string): Promise<string> {
	return tauriInvoke<string>('read_artifact', { projectId, id });
}

export async function updateArtifact(
	projectId: string,
	id: string,
	content: string
): Promise<Artifact> {
	return tauriInvoke<Artifact>('update_artifact', { projectId, id, content });
}

export async function deleteArtifact(projectId: string, id: string): Promise<void> {
	return tauriInvoke<void>('delete_artifact', { projectId, id });
}

export async function listArtifacts(projectId: string, cardId: string): Promise<Artifact[]> {
	return tauriInvoke<Artifact[]>('list_artifacts', { projectId, cardId });
}
