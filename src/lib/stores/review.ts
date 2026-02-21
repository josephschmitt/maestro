import { writable, derived, get } from 'svelte/store';
import type { ChangedFile, FileDiff } from '$lib/types/index.js';
import {
	getChangedFiles as getChangedFilesService,
	getFileDiff as getFileDiffService,
	sendBackCard as sendBackCardService,
	approveCard as approveCardService,
	createPr as createPrService,
	getReviewCount as getReviewCountService
} from '$lib/services/review.js';
import { currentProject } from './project.js';

export const changedFiles = writable<ChangedFile[]>([]);
export const selectedFilePath = writable<string | null>(null);
export const fileDiff = writable<FileDiff | null>(null);
export const reviewCount = writable<number>(0);
export const reviewLoading = writable<boolean>(false);
export const reviewError = writable<string | null>(null);

export const selectedFile = derived(
	[changedFiles, selectedFilePath],
	([$changedFiles, $selectedFilePath]) =>
		$changedFiles.find((f) => f.path === $selectedFilePath) ?? null
);

export async function loadChangedFiles(cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;

	reviewLoading.set(true);
	reviewError.set(null);

	try {
		const files = await getChangedFilesService(project.id, cardId);
		changedFiles.set(files);
		selectedFilePath.set(null);
		fileDiff.set(null);
	} catch (e) {
		reviewError.set(e instanceof Error ? e.message : String(e));
		changedFiles.set([]);
	} finally {
		reviewLoading.set(false);
	}
}

export async function selectFile(cardId: string, path: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;

	selectedFilePath.set(path);
	reviewLoading.set(true);

	try {
		const diff = await getFileDiffService(project.id, cardId, path);
		fileDiff.set(diff);
	} catch (e) {
		reviewError.set(e instanceof Error ? e.message : String(e));
		fileDiff.set(null);
	} finally {
		reviewLoading.set(false);
	}
}

export async function sendBack(
	cardId: string,
	feedback: string,
	inProgressStatusId: string
): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await sendBackCardService(project.id, cardId, feedback, inProgressStatusId);
}

export async function approve(
	cardId: string,
	completedStatusId: string
): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await approveCardService(project.id, cardId, completedStatusId);
}

export async function openPr(
	cardId: string,
	title: string,
	body: string
): Promise<string> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const result = await createPrService(project.id, cardId, title, body);
	return result.url;
}

export async function loadReviewCount(cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	try {
		const count = await getReviewCountService(project.id, cardId);
		reviewCount.set(count);
	} catch {
		reviewCount.set(0);
	}
}

export function resetReviewState(): void {
	changedFiles.set([]);
	selectedFilePath.set(null);
	fileDiff.set(null);
	reviewCount.set(0);
	reviewLoading.set(false);
	reviewError.set(null);
}
