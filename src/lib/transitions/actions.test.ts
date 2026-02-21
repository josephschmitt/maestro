import { describe, it, expect, vi } from 'vitest';
import { executeActions, type ActionContext } from './actions.js';
import type { TransitionAction } from './engine.js';

function makeContext(overrides: Partial<ActionContext> = {}): ActionContext {
	return {
		cardId: 'card-1',
		cardTitle: 'Test Card',
		showLinkDirectoryPrompt: vi.fn(),
		runWorktreeFlow: vi.fn().mockResolvedValue(null),
		archiveWorkspaces: vi.fn().mockResolvedValue(undefined),
		stopAgent: vi.fn().mockResolvedValue(undefined),
		...overrides
	};
}

describe('executeActions', () => {
	it('calls showLinkDirectoryPrompt for show-link-directory-prompt action', async () => {
		const ctx = makeContext();
		const actions: TransitionAction[] = [{ type: 'show-link-directory-prompt' }];
		await executeActions(actions, ctx);
		expect(ctx.showLinkDirectoryPrompt).toHaveBeenCalled();
	});

	it('calls runWorktreeFlow for run-worktree-flow action', async () => {
		const ctx = makeContext();
		const actions: TransitionAction[] = [{ type: 'run-worktree-flow' }];
		await executeActions(actions, ctx);
		expect(ctx.runWorktreeFlow).toHaveBeenCalledWith('card-1', 'Test Card');
	});

	it('returns worktree result from run-worktree-flow', async () => {
		const worktreeResult = { worktreePath: '/tmp/wt', branchName: 'feat-1', repoPath: '/repo' };
		const ctx = makeContext({
			runWorktreeFlow: vi.fn().mockResolvedValue(worktreeResult)
		});
		const actions: TransitionAction[] = [{ type: 'run-worktree-flow' }];
		const result = await executeActions(actions, ctx);
		expect(result).toEqual(worktreeResult);
	});

	it('calls archiveWorkspaces for archive-workspaces action', async () => {
		const ctx = makeContext();
		const actions: TransitionAction[] = [{ type: 'archive-workspaces' }];
		await executeActions(actions, ctx);
		expect(ctx.archiveWorkspaces).toHaveBeenCalledWith('card-1');
	});

	it('stops agent when runningAgentChoice is stop', async () => {
		const ctx = makeContext();
		const actions: TransitionAction[] = [];
		await executeActions(actions, ctx, 'stop');
		expect(ctx.stopAgent).toHaveBeenCalledWith('card-1');
	});

	it('does not stop agent when choice is keep-running', async () => {
		const ctx = makeContext();
		const actions: TransitionAction[] = [];
		await executeActions(actions, ctx, 'keep-running');
		expect(ctx.stopAgent).not.toHaveBeenCalled();
	});

	it('executes multiple actions in order', async () => {
		const callOrder: string[] = [];
		const ctx = makeContext({
			archiveWorkspaces: vi.fn().mockImplementation(async () => {
				callOrder.push('archive');
			}),
			runWorktreeFlow: vi.fn().mockImplementation(async () => {
				callOrder.push('worktree');
				return null;
			})
		});
		const actions: TransitionAction[] = [
			{ type: 'archive-workspaces' },
			{ type: 'run-worktree-flow' }
		];
		await executeActions(actions, ctx);
		expect(callOrder).toEqual(['archive', 'worktree']);
	});
});
