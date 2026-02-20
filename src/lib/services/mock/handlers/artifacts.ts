import type { Artifact } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

function nameToSlug(name: string): string {
	return name
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '');
}

export function create_artifact(args: Record<string, unknown>): Artifact {
	const store = getStore();
	const cardId = args.cardId as string;
	const name = args.name as string;
	const slug = nameToSlug(name);

	const artifact: Artifact = {
		id: newId(),
		card_id: cardId,
		name,
		type: 'markdown',
		path: `artifacts/${cardId}/${slug}.md`,
		created_by: (args.createdBy as 'user' | 'agent') ?? 'user',
		created_at: nowISO(),
		updated_at: nowISO()
	};
	store.artifacts.push(artifact);

	const content = (args.content as string) ?? '';
	store.artifactContents.set(artifact.id, content);

	return artifact;
}

export function read_artifact(args: Record<string, unknown>): string {
	const store = getStore();
	const id = args.id as string;
	return store.artifactContents.get(id) ?? '';
}

export function update_artifact(args: Record<string, unknown>): Artifact {
	const store = getStore();
	const id = args.id as string;
	const content = args.content as string;

	const artifact = store.artifacts.find((a) => a.id === id);
	if (!artifact) throw new Error(`Artifact not found: ${id}`);

	artifact.updated_at = nowISO();
	store.artifactContents.set(id, content);

	return artifact;
}

export function delete_artifact(args: Record<string, unknown>): void {
	const store = getStore();
	const id = args.id as string;
	store.artifacts = store.artifacts.filter((a) => a.id !== id);
	store.artifactContents.delete(id);
}

export function list_artifacts(args: Record<string, unknown>): Artifact[] {
	const store = getStore();
	return store.artifacts
		.filter((a) => a.card_id === args.cardId)
		.sort((a, b) => b.created_at.localeCompare(a.created_at));
}
