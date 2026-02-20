import * as projects from './handlers/projects.js';
import * as statuses from './handlers/statuses.js';
import * as cards from './handlers/cards.js';
import * as config from './handlers/config.js';
import * as questions from './handlers/questions.js';

type Handler = (args: Record<string, unknown>) => unknown;

const handlers: Record<string, Handler> = {
	create_project: projects.create_project,
	get_project: projects.get_project,
	list_projects: projects.list_projects,
	update_project: projects.update_project,
	delete_project: projects.delete_project,

	list_statuses: statuses.list_statuses,
	create_status: statuses.create_status,
	update_status: statuses.update_status,
	delete_status: statuses.delete_status,
	reorder_statuses: statuses.reorder_statuses,

	create_card: cards.create_card,
	get_card: cards.get_card,
	update_card: cards.update_card,
	delete_card: cards.delete_card,
	list_cards: cards.list_cards,
	list_sub_cards: cards.list_sub_cards,
	move_card: cards.move_card,
	reorder_cards: cards.reorder_cards,

	get_global_config: config.get_global_config,
	set_last_project: config.set_last_project,
	resolve_config: config.resolve_config,

	create_question: questions.create_question,
	list_questions: questions.list_questions,
	resolve_question: questions.resolve_question,
	unresolve_question: questions.unresolve_question,
	delete_question: questions.delete_question,
	count_unresolved_questions: questions.count_unresolved_questions
};

export function dispatchMockCommand<T>(command: string, args?: Record<string, unknown>): T {
	const handler = handlers[command];
	if (!handler) {
		console.warn(`[mock] Unhandled command: ${command}`);
		return undefined as T;
	}
	return handler(args ?? {}) as T;
}
