export type { StatusGroup } from './status.js';
export { STATUS_GROUPS } from './status.js';
export type { CardWithStatus, CardProgress } from './card.js';

export interface Project {
	id: string;
	name: string;
	agent_config: Record<string, unknown>;
	base_path: string | null;
	created_at: string;
	updated_at: string;
}

export interface LinkedDirectory {
	id: string;
	project_id: string;
	path: string;
	label: string;
	is_repo: boolean;
	created_at: string;
}

export interface Status {
	id: string;
	project_id: string;
	group: import('./status.js').StatusGroup;
	name: string;
	sort_order: number;
	is_default: boolean;
	status_prompts: string[];
	created_at: string;
}

export interface Card {
	id: string;
	project_id: string;
	parent_id: string | null;
	status_id: string;
	title: string;
	description: string;
	labels: string[];
	sort_order: number;
	created_at: string;
	updated_at: string;
}

export interface OpenQuestion {
	id: string;
	card_id: string;
	question: string;
	resolution: string | null;
	source: 'agent' | 'user';
	resolved_by: 'agent' | 'user' | null;
	created_at: string;
	resolved_at: string | null;
}

export interface Conversation {
	id: string;
	card_id: string;
	agent_type: string;
	started_at: string;
	ended_at: string | null;
}

export interface ConversationMessage {
	id: string;
	conversation_id: string;
	role: 'user' | 'agent';
	content: string;
	timestamp: string;
}

export type AgentWorkspaceStatus = 'running' | 'paused' | 'reviewing' | 'completed' | 'failed';

export interface AgentWorkspace {
	id: string;
	card_id: string;
	agent_type: string;
	status: AgentWorkspaceStatus;
	session_id: string | null;
	pid: number | null;
	worktree_path: string | null;
	branch_name: string | null;
	review_count: number;
	attached_at: string;
	completed_at: string | null;
}

export interface Artifact {
	id: string;
	card_id: string;
	name: string;
	type: string;
	path: string;
	created_by: 'user' | 'agent';
	created_at: string;
	updated_at: string;
}

export interface ProjectSummary {
	id: string;
	name: string;
	created_at: string;
}

export interface AgentProfileConfig {
	name: string;
	binary: string;
	flags: string[];
	custom_command: string | null;
	env_vars: Record<string, string> | null;
}

export interface AgentProfileInput {
	name: string;
	binary: string;
	flags: string[];
	custom_command: string | null;
	env_vars: Record<string, string> | null;
}

export interface GlobalConfigUpdate {
	storage_base_path?: string;
	default_agent?: string;
}

export interface StatusGroupConfigInput {
	agent: string | null;
	model: string | null;
	instructions: string | null;
}

export interface StatusGroupDefaultsResponse {
	status: Record<string, StatusGroupConfigInput>;
}

export interface GlobalConfigResponse {
	storage_base_path: string;
	default_agent: string;
	last_project_id: string;
	agents: AgentProfileConfig[];
}

export interface ResolvedAgentConfigResponse {
	agent: string;
	model: string | null;
	instructions: string | null;
}

export type FileChangeStatus = 'A' | 'M' | 'D';

export interface ChangedFile {
	path: string;
	status: FileChangeStatus;
}

export type DiffLineType = 'added' | 'removed' | 'context';

export interface DiffLine {
	line_type: DiffLineType;
	content: string;
	old_line: number | null;
	new_line: number | null;
}

export interface DiffHunk {
	old_start: number;
	old_count: number;
	new_start: number;
	new_count: number;
	header: string;
	lines: DiffLine[];
}

export interface FileDiff {
	path: string;
	hunks: DiffHunk[];
}

export interface CreatePrResult {
	url: string;
}

export interface HttpServerConfigResponse {
	enabled: boolean;
	port: number;
	bind_address: string;
	auth_token: string;
	requires_auth: boolean;
	server_url: string;
}

export interface HttpServerConfigUpdate {
	enabled?: boolean;
	port?: number;
	bind_address?: string;
}
