import type { ChangedFile, FileDiff, CreatePrResult } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function getChangedFiles(
	projectId: string,
	cardId: string
): Promise<ChangedFile[]> {
	return tauriInvoke<ChangedFile[]>('get_changed_files', { projectId, cardId });
}

export async function getFileDiff(
	projectId: string,
	cardId: string,
	filePath: string
): Promise<FileDiff> {
	return tauriInvoke<FileDiff>('get_file_diff', { projectId, cardId, filePath });
}

export async function sendBackCard(
	projectId: string,
	cardId: string,
	feedback: string,
	inProgressStatusId: string
): Promise<void> {
	return tauriInvoke<void>('send_back_card', {
		projectId,
		cardId,
		feedback,
		inProgressStatusId
	});
}

export async function approveCard(
	projectId: string,
	cardId: string,
	completedStatusId: string
): Promise<void> {
	return tauriInvoke<void>('approve_card', { projectId, cardId, completedStatusId });
}

export async function createPr(
	projectId: string,
	cardId: string,
	title: string,
	body: string
): Promise<CreatePrResult> {
	return tauriInvoke<CreatePrResult>('create_pr', { projectId, cardId, title, body });
}

export async function getReviewCount(
	projectId: string,
	cardId: string
): Promise<number> {
	return tauriInvoke<number>('get_review_count', { projectId, cardId });
}
