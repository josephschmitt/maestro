import { describe, it, expect } from 'vitest';
import { getTransitionPlan, type TransitionContext } from './engine.js';

function makeContext(overrides: Partial<TransitionContext> = {}): TransitionContext {
	return {
		fromGroup: 'Backlog',
		toGroup: 'Unstarted',
		hasLinkedDirs: true,
		unresolvedQuestionCount: 0,
		hasRunningAgent: false,
		...overrides
	};
}

describe('getTransitionPlan', () => {
	it('returns empty plan for same-group moves', () => {
		const plan = getTransitionPlan(makeContext({ fromGroup: 'Started', toGroup: 'Started' }));
		expect(plan.gates).toEqual([]);
		expect(plan.actions).toEqual([]);
	});

	describe('Backlog → Unstarted', () => {
		it('returns linked-dirs gate when no dirs linked', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Backlog', toGroup: 'Unstarted', hasLinkedDirs: false })
			);
			expect(plan.gates).toHaveLength(1);
			expect(plan.gates[0].id).toBe('linked-dirs');
			expect(plan.gates[0].type).toBe('soft');
			expect(plan.actions).toEqual([{ type: 'show-link-directory-prompt' }]);
		});

		it('returns empty gates when dirs are linked', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Backlog', toGroup: 'Unstarted', hasLinkedDirs: true })
			);
			expect(plan.gates).toHaveLength(0);
			expect(plan.actions).toHaveLength(0);
		});
	});

	describe('Unstarted → Started', () => {
		it('returns open-questions gate and worktree action when questions exist', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Unstarted', toGroup: 'Started', unresolvedQuestionCount: 3 })
			);
			expect(plan.gates).toHaveLength(1);
			expect(plan.gates[0].id).toBe('open-questions');
			expect(plan.gates[0].type).toBe('soft');
			if (plan.gates[0].id === 'open-questions') {
				expect(plan.gates[0].unresolvedCount).toBe(3);
			}
			expect(plan.actions).toContainEqual({ type: 'run-worktree-flow' });
		});

		it('returns only worktree action when no questions', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Unstarted', toGroup: 'Started', unresolvedQuestionCount: 0 })
			);
			expect(plan.gates).toHaveLength(0);
			expect(plan.actions).toEqual([{ type: 'run-worktree-flow' }]);
		});
	});

	describe('backward moves with running agent', () => {
		it('returns running-agent gate when moving backward with agent', () => {
			const plan = getTransitionPlan(
				makeContext({
					fromGroup: 'Started',
					toGroup: 'Unstarted',
					hasRunningAgent: true
				})
			);
			expect(plan.gates).toHaveLength(1);
			expect(plan.gates[0].id).toBe('running-agent');
			expect(plan.gates[0].type).toBe('prompt');
		});

		it('returns no running-agent gate when no agent running', () => {
			const plan = getTransitionPlan(
				makeContext({
					fromGroup: 'Started',
					toGroup: 'Unstarted',
					hasRunningAgent: false
				})
			);
			expect(plan.gates.find((g) => g.id === 'running-agent')).toBeUndefined();
		});
	});

	describe('Completed → earlier', () => {
		it('returns archive-workspaces action', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Completed', toGroup: 'Unstarted' })
			);
			expect(plan.actions).toContainEqual({ type: 'archive-workspaces' });
		});

		it('includes running-agent gate if agent is running', () => {
			const plan = getTransitionPlan(
				makeContext({
					fromGroup: 'Completed',
					toGroup: 'Unstarted',
					hasRunningAgent: true
				})
			);
			expect(plan.gates.find((g) => g.id === 'running-agent')).toBeDefined();
			expect(plan.actions).toContainEqual({ type: 'archive-workspaces' });
		});
	});

	describe('Cancelled → earlier', () => {
		it('returns archive-workspaces action', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Cancelled', toGroup: 'Started' })
			);
			expect(plan.actions).toContainEqual({ type: 'archive-workspaces' });
		});
	});

	describe('forward moves without special handling', () => {
		it('Started → Completed returns empty plan', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Started', toGroup: 'Completed' })
			);
			expect(plan.gates).toHaveLength(0);
			expect(plan.actions).toHaveLength(0);
		});

		it('Started → Cancelled returns empty plan', () => {
			const plan = getTransitionPlan(
				makeContext({ fromGroup: 'Started', toGroup: 'Cancelled' })
			);
			expect(plan.gates).toHaveLength(0);
			expect(plan.actions).toHaveLength(0);
		});
	});

	describe('Backlog → Started (skipping Unstarted)', () => {
		it('returns open-questions gate and worktree action', () => {
			const plan = getTransitionPlan(
				makeContext({
					fromGroup: 'Backlog',
					toGroup: 'Started',
					unresolvedQuestionCount: 2
				})
			);
			expect(plan.gates).toHaveLength(1);
			expect(plan.gates[0].id).toBe('open-questions');
			expect(plan.actions).toContainEqual({ type: 'run-worktree-flow' });
		});
	});
});
