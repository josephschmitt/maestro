export { getTransitionPlan } from './engine.js';
export type {
	Gate,
	GateType,
	TransitionGate,
	OpenQuestionsGate,
	LinkedDirsGate,
	RunningAgentGate,
	TransitionAction,
	ActionType,
	TransitionPlan,
	TransitionContext
} from './engine.js';

export { gatherTransitionContext } from './gates.js';
export type {
	RunningAgentChoice,
	GateResolution,
	GateCheckResult,
	GateDataSources
} from './gates.js';

export { executeActions } from './actions.js';
export type { ActionContext, WorktreeResult } from './actions.js';
