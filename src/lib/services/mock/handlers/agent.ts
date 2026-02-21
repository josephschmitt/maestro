import type { AgentWorkspace } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

export function launch_agent(args: Record<string, unknown>): AgentWorkspace {
	const store = getStore();
	const worktreePath = (args.worktreePath as string | null) ?? null;
	const branchName = (args.branchName as string | null) ?? null;
	const workspace: AgentWorkspace = {
		id: newId(),
		card_id: args.cardId as string,
		agent_type: 'claude-code',
		status: 'running',
		session_id: null,
		pid: Math.floor(Math.random() * 90000) + 10000,
		worktree_path: worktreePath,
		branch_name: branchName,
		review_count: 0,
		attached_at: nowISO(),
		completed_at: null
	};
	store.agentWorkspaces.push(workspace);

	simulateMockOutput(workspace.id);

	return workspace;
}

export function send_agent_input(_args: Record<string, unknown>): void {
	// No-op in mock mode
}

export function stop_agent(args: Record<string, unknown>): AgentWorkspace {
	const store = getStore();
	const workspaceId = args.workspaceId as string;
	const ws = store.agentWorkspaces.find((w) => w.id === workspaceId);
	if (ws) {
		ws.status = 'failed';
		ws.completed_at = nowISO();
	}
	return ws!;
}

export function list_workspaces(args: Record<string, unknown>): AgentWorkspace[] {
	const store = getStore();
	return store.agentWorkspaces
		.filter((w) => w.card_id === args.cardId)
		.sort((a, b) => b.attached_at.localeCompare(a.attached_at));
}

export function get_workspace(args: Record<string, unknown>): AgentWorkspace | undefined {
	const store = getStore();
	return store.agentWorkspaces.find((w) => w.id === args.workspaceId);
}

export function resume_agent(args: Record<string, unknown>): AgentWorkspace {
	const store = getStore();
	const oldWs = store.agentWorkspaces.find((w) => w.id === args.workspaceId);
	const workspace: AgentWorkspace = {
		id: newId(),
		card_id: (args.cardId as string) || oldWs?.card_id || '',
		agent_type: oldWs?.agent_type || 'claude-code',
		status: 'running',
		session_id: oldWs?.session_id,
		pid: Math.floor(Math.random() * 90000) + 10000,
		worktree_path: oldWs?.worktree_path ?? null,
		branch_name: oldWs?.branch_name ?? null,
		review_count: 0,
		attached_at: nowISO(),
		completed_at: null
	};
	store.agentWorkspaces.push(workspace);
	simulateMockOutput(workspace.id);
	return workspace;
}

export function list_running_workspaces(): AgentWorkspace[] {
	const store = getStore();
	return store.agentWorkspaces.filter((w) => w.status === 'running');
}

export function stop_all_agents(): void {
	const store = getStore();
	for (const ws of store.agentWorkspaces) {
		if (ws.status === 'running') {
			ws.status = 'failed';
			ws.completed_at = nowISO();
		}
	}
}

function simulateMockOutput(workspaceId: string): void {
	const store = getStore();
	const lines = [
		'Resolving agent configuration...',
		'Starting exploration session...',
		'Reading card context and artifacts...',
		'Agent ready. Waiting for instructions.'
	];

	let i = 0;
	const interval = setInterval(() => {
		if (i >= lines.length) {
			clearInterval(interval);
			const ws = store.agentWorkspaces.find((w) => w.id === workspaceId);
			if (ws && ws.status === 'running') {
				ws.status = 'completed';
				ws.completed_at = nowISO();
			}
			return;
		}
		i++;
	}, 500);
}
