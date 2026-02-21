import type { OpenQuestion } from '$lib/types/index.js';
import type { TransitionContext, TransitionGate } from './engine.js';

export type RunningAgentChoice = 'stop' | 'keep-running' | 'cancel';

export interface GateResolution {
	gate: TransitionGate;
	proceed: boolean;
	choice?: RunningAgentChoice;
}

export interface GateCheckResult {
	proceed: boolean;
	resolutions: GateResolution[];
}

export interface GateDataSources {
	getUnresolvedQuestions: (cardId: string) => Promise<OpenQuestion[]>;
	getLinkedDirCount: () => number;
	getRunningWorkspaceForCard: (cardId: string) => Promise<boolean>;
}

export async function gatherTransitionContext(
	fromGroup: TransitionContext['fromGroup'],
	toGroup: TransitionContext['toGroup'],
	cardId: string,
	sources: GateDataSources
): Promise<TransitionContext> {
	const [questions, hasRunningAgent] = await Promise.all([
		sources.getUnresolvedQuestions(cardId),
		sources.getRunningWorkspaceForCard(cardId)
	]);

	return {
		fromGroup,
		toGroup,
		hasLinkedDirs: sources.getLinkedDirCount() > 0,
		unresolvedQuestionCount: questions.length,
		hasRunningAgent
	};
}
