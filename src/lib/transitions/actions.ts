import type { TransitionAction } from './engine.js';
import type { RunningAgentChoice } from './gates.js';

export interface ActionContext {
	cardId: string;
	cardTitle: string;
	showLinkDirectoryPrompt: () => void;
	runWorktreeFlow: (cardId: string, cardTitle: string) => Promise<WorktreeResult | null>;
	archiveWorkspaces: (cardId: string) => Promise<void>;
	stopAgent: (cardId: string) => Promise<void>;
}

export interface WorktreeResult {
	worktreePath: string;
	branchName: string;
	repoPath: string;
}

export async function executeActions(
	actions: TransitionAction[],
	ctx: ActionContext,
	runningAgentChoice?: RunningAgentChoice
): Promise<WorktreeResult | null> {
	let worktreeResult: WorktreeResult | null = null;

	if (runningAgentChoice === 'stop') {
		await ctx.stopAgent(ctx.cardId);
	}

	for (const action of actions) {
		switch (action.type) {
			case 'show-link-directory-prompt':
				ctx.showLinkDirectoryPrompt();
				break;
			case 'run-worktree-flow':
				worktreeResult = await ctx.runWorktreeFlow(ctx.cardId, ctx.cardTitle);
				break;
			case 'archive-workspaces':
				await ctx.archiveWorkspaces(ctx.cardId);
				break;
		}
	}

	return worktreeResult;
}
