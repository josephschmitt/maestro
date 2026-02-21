import * as projects from './handlers/projects.js';
import * as statuses from './handlers/statuses.js';
import * as cards from './handlers/cards.js';
import * as config from './handlers/config.js';
import * as questions from './handlers/questions.js';
import * as artifacts from './handlers/artifacts.js';
import * as directories from './handlers/directories.js';
import * as conversations from './handlers/conversations.js';
import * as agent from './handlers/agent.js';
import * as worktrees from './handlers/worktrees.js';

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
	count_unresolved_questions: questions.count_unresolved_questions,

	create_artifact: artifacts.create_artifact,
	read_artifact: artifacts.read_artifact,
	update_artifact: artifacts.update_artifact,
	delete_artifact: artifacts.delete_artifact,
	list_artifacts: artifacts.list_artifacts,

	add_linked_directory: directories.add_linked_directory,
	remove_linked_directory: directories.remove_linked_directory,
	list_linked_directories: directories.list_linked_directories,

	create_conversation: conversations.create_conversation,
	list_conversations: conversations.list_conversations,
	create_message: conversations.create_message,
	list_messages: conversations.list_messages,
	count_conversation_messages: conversations.count_conversation_messages,

	launch_agent: agent.launch_agent,
	send_agent_input: agent.send_agent_input,
	stop_agent: agent.stop_agent,
	resume_agent: agent.resume_agent,
	list_workspaces: agent.list_workspaces,
	get_workspace: agent.get_workspace,
	list_running_workspaces: agent.list_running_workspaces,
	stop_all_agents: agent.stop_all_agents,

	generate_branch_name: worktrees.generate_branch_name,
	create_worktree: worktrees.create_worktree,
	check_worktree_exists: worktrees.check_worktree_exists,
	get_card_worktree: worktrees.get_card_worktree,
	get_claude_worktree_path: worktrees.get_claude_worktree_path
};

export function dispatchMockCommand<T>(command: string, args?: Record<string, unknown>): T {
	const handler = handlers[command];
	if (!handler) {
		console.warn(`[mock] Unhandled command: ${command}`);
		return undefined as T;
	}
	return handler(args ?? {}) as T;
}
