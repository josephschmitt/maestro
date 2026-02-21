import type { StatusGroup } from '$lib/types/index.js';

export type GateType = 'soft' | 'prompt';

export interface Gate {
	type: GateType;
	id: string;
	title: string;
	description: string;
}

export interface OpenQuestionsGate extends Gate {
	id: 'open-questions';
	type: 'soft';
	unresolvedCount: number;
}

export interface LinkedDirsGate extends Gate {
	id: 'linked-dirs';
	type: 'soft';
}

export interface RunningAgentGate extends Gate {
	id: 'running-agent';
	type: 'prompt';
}

export type TransitionGate = OpenQuestionsGate | LinkedDirsGate | RunningAgentGate;

export type ActionType =
	| 'show-link-directory-prompt'
	| 'run-worktree-flow'
	| 'archive-workspaces';

export interface TransitionAction {
	type: ActionType;
}

export interface TransitionPlan {
	gates: TransitionGate[];
	actions: TransitionAction[];
}

export interface TransitionContext {
	fromGroup: StatusGroup;
	toGroup: StatusGroup;
	hasLinkedDirs: boolean;
	unresolvedQuestionCount: number;
	hasRunningAgent: boolean;
}

function getGroupIndex(group: StatusGroup): number {
	const order: StatusGroup[] = ['Backlog', 'Unstarted', 'Started', 'Completed', 'Cancelled'];
	return order.indexOf(group);
}

function isBackwardMove(from: StatusGroup, to: StatusGroup): boolean {
	if (from === 'Cancelled' || from === 'Completed') return true;
	return getGroupIndex(to) < getGroupIndex(from);
}

export function getTransitionPlan(ctx: TransitionContext): TransitionPlan {
	const { fromGroup, toGroup, hasLinkedDirs, unresolvedQuestionCount, hasRunningAgent } = ctx;

	if (fromGroup === toGroup) {
		return { gates: [], actions: [] };
	}

	const gates: TransitionGate[] = [];
	const actions: TransitionAction[] = [];

	const backward = isBackwardMove(fromGroup, toGroup);

	if (backward && hasRunningAgent) {
		gates.push({
			id: 'running-agent',
			type: 'prompt',
			title: 'Running Agent',
			description: 'An agent is currently running on this card.'
		});
	}

	if ((fromGroup === 'Completed' || fromGroup === 'Cancelled') && backward) {
		actions.push({ type: 'archive-workspaces' });
	}

	if (toGroup === 'Unstarted' && fromGroup === 'Backlog') {
		if (!hasLinkedDirs) {
			gates.push({
				id: 'linked-dirs',
				type: 'soft',
				title: 'No Linked Directories',
				description:
					'This project has no linked directories. You may want to link one before starting work.'
			});
			actions.push({ type: 'show-link-directory-prompt' });
		}
	}

	if (toGroup === 'Started' && !backward) {
		if (unresolvedQuestionCount > 0) {
			gates.push({
				id: 'open-questions',
				type: 'soft',
				title: 'Unresolved Questions',
				description: `There are ${unresolvedQuestionCount} unresolved question${unresolvedQuestionCount === 1 ? '' : 's'} on this card.`,
				unresolvedCount: unresolvedQuestionCount
			});
		}
		actions.push({ type: 'run-worktree-flow' });
	}

	return { gates, actions };
}
