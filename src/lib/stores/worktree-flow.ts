import { writable, get } from 'svelte/store';
import type { LinkedDirectory } from '$lib/types/index.js';
import { linkedDirectories } from './directories.js';
import { currentProject } from './project.js';
import {
	generateBranchName,
	createWorktree,
	getCardWorktree
} from '$lib/services/worktrees.js';

export interface WorktreeFlowResult {
	worktreePath: string;
	branchName: string;
}

interface RepoSelectorState {
	open: boolean;
	repos: LinkedDirectory[];
	resolve: ((repo: LinkedDirectory | null) => void) | null;
}

interface BranchNameState {
	open: boolean;
	defaultBranchName: string;
	resolve: ((branchName: string | null) => void) | null;
}

export const repoSelectorState = writable<RepoSelectorState>({
	open: false,
	repos: [],
	resolve: null
});

export const branchNameState = writable<BranchNameState>({
	open: false,
	defaultBranchName: '',
	resolve: null
});

function selectRepo(repos: LinkedDirectory[]): Promise<LinkedDirectory | null> {
	return new Promise((resolve) => {
		repoSelectorState.set({ open: true, repos, resolve });
	});
}

function confirmBranchName(defaultName: string): Promise<string | null> {
	return new Promise((resolve) => {
		branchNameState.set({ open: true, defaultBranchName: defaultName, resolve });
	});
}

export function resolveRepoSelection(repo: LinkedDirectory | null): void {
	const state = get(repoSelectorState);
	if (state.resolve) {
		state.resolve(repo);
	}
	repoSelectorState.set({ open: false, repos: [], resolve: null });
}

export function resolveBranchName(branchName: string | null): void {
	const state = get(branchNameState);
	if (state.resolve) {
		state.resolve(branchName);
	}
	branchNameState.set({ open: false, defaultBranchName: '', resolve: null });
}

export async function runWorktreeFlow(
	cardId: string,
	cardTitle: string
): Promise<WorktreeFlowResult | null> {
	const project = get(currentProject);
	if (!project) return null;

	const existing = await getCardWorktree(project.id, cardId);
	if (existing) {
		return { worktreePath: existing.path, branchName: existing.branch };
	}

	const dirs = get(linkedDirectories);
	const repos = dirs.filter((d) => d.is_repo);

	let selectedRepo: LinkedDirectory | null = null;

	if (repos.length === 0) {
		return null;
	} else if (repos.length === 1) {
		selectedRepo = repos[0];
	} else {
		selectedRepo = await selectRepo(repos);
		if (!selectedRepo) {
			return null;
		}
	}

	const defaultBranch = await generateBranchName(cardId, cardTitle);
	const branchName = await confirmBranchName(defaultBranch);
	if (!branchName) {
		return null;
	}

	const worktreePath = await createWorktree(
		project.id,
		cardId,
		selectedRepo.path,
		branchName
	);

	return { worktreePath, branchName };
}
