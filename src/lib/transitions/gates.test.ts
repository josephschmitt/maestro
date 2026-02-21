import { describe, it, expect, vi } from 'vitest';
import { gatherTransitionContext, type GateDataSources } from './gates.js';

function makeSources(overrides: Partial<GateDataSources> = {}): GateDataSources {
	return {
		getUnresolvedQuestions: vi.fn().mockResolvedValue([]),
		getLinkedDirCount: vi.fn().mockReturnValue(1),
		getRunningWorkspaceForCard: vi.fn().mockResolvedValue(false),
		...overrides
	};
}

describe('gatherTransitionContext', () => {
	it('returns context with correct groups', async () => {
		const ctx = await gatherTransitionContext('Backlog', 'Started', 'card-1', makeSources());
		expect(ctx.fromGroup).toBe('Backlog');
		expect(ctx.toGroup).toBe('Started');
	});

	it('counts unresolved questions', async () => {
		const sources = makeSources({
			getUnresolvedQuestions: vi.fn().mockResolvedValue([
				{ id: 'q1', question: 'Question 1' },
				{ id: 'q2', question: 'Question 2' }
			])
		});
		const ctx = await gatherTransitionContext('Unstarted', 'Started', 'card-1', sources);
		expect(ctx.unresolvedQuestionCount).toBe(2);
	});

	it('checks linked dirs', async () => {
		const sources = makeSources({
			getLinkedDirCount: vi.fn().mockReturnValue(0)
		});
		const ctx = await gatherTransitionContext('Backlog', 'Unstarted', 'card-1', sources);
		expect(ctx.hasLinkedDirs).toBe(false);
	});

	it('checks running agent', async () => {
		const sources = makeSources({
			getRunningWorkspaceForCard: vi.fn().mockResolvedValue(true)
		});
		const ctx = await gatherTransitionContext('Started', 'Backlog', 'card-1', sources);
		expect(ctx.hasRunningAgent).toBe(true);
	});

	it('calls data sources with correct card ID', async () => {
		const sources = makeSources();
		await gatherTransitionContext('Backlog', 'Started', 'card-42', sources);
		expect(sources.getUnresolvedQuestions).toHaveBeenCalledWith('card-42');
		expect(sources.getRunningWorkspaceForCard).toHaveBeenCalledWith('card-42');
	});
});
