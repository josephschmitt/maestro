import { tauriInvoke } from './db.js';

export interface WorktreeInfo {
	path: string;
	branch: string;
}

export async function generateBranchName(cardId: string, title: string): Promise<string> {
	return tauriInvoke<string>('generate_branch_name', { cardId, title });
}

export async function createWorktree(
	projectId: string,
	cardId: string,
	repoPath: string,
	branchName: string
): Promise<string> {
	return tauriInvoke<string>('create_worktree', { projectId, cardId, repoPath, branchName });
}

export async function checkWorktreeExists(
	projectId: string,
	cardId: string,
	branchSlug: string
): Promise<string | null> {
	return tauriInvoke<string | null>('check_worktree_exists', {
		projectId,
		cardId,
		branchSlug
	});
}

export async function getCardWorktree(
	projectId: string,
	cardId: string
): Promise<WorktreeInfo | null> {
	return tauriInvoke<WorktreeInfo | null>('get_card_worktree', { projectId, cardId });
}
