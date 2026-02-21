import { getStore } from '../store.js';

function nameToSlug(name: string): string {
	return name
		.toLowerCase()
		.replace(/[^a-z0-9]/g, '-')
		.replace(/-+/g, '-')
		.replace(/^-|-$/g, '');
}

export function generate_branch_name(args: Record<string, unknown>): string {
	const cardId = args.cardId as string;
	const title = args.title as string;
	const cardShort = cardId.slice(0, 8);
	const slug = nameToSlug(title).slice(0, 40).replace(/-$/, '');
	return `maestro/${cardShort}-${slug}`;
}

export function create_worktree(args: Record<string, unknown>): string {
	const cardId = args.cardId as string;
	const branchName = args.branchName as string;
	const cardShort = cardId.slice(0, 8);
	const slug = branchName.replace('maestro/', '');
	return `/tmp/maestro/projects/mock/worktrees/${cardShort}-${slug}`;
}

export function check_worktree_exists(
	_args: Record<string, unknown>
): string | null {
	return null;
}

export function get_card_worktree(
	args: Record<string, unknown>
): { path: string; branch: string } | null {
	const store = getStore();
	const cardId = args.cardId as string;
	const ws = store.agentWorkspaces.find(
		(w) => w.card_id === cardId && w.worktree_path != null
	);
	if (ws && ws.worktree_path && ws.branch_name) {
		return { path: ws.worktree_path, branch: ws.branch_name };
	}
	return null;
}
