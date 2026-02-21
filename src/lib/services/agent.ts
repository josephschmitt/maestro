import type { AgentWorkspace } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function launchAgent(
	projectId: string,
	cardId: string,
	statusGroup: string,
	worktreePath?: string | null,
	branchName?: string | null
): Promise<AgentWorkspace> {
	return tauriInvoke<AgentWorkspace>('launch_agent', {
		projectId,
		cardId,
		statusGroup,
		worktreePath: worktreePath ?? null,
		branchName: branchName ?? null
	});
}

export async function sendAgentInput(workspaceId: string, text: string): Promise<void> {
	return tauriInvoke<void>('send_agent_input', { workspaceId, text });
}

export async function stopAgent(projectId: string, workspaceId: string): Promise<AgentWorkspace> {
	return tauriInvoke<AgentWorkspace>('stop_agent', { projectId, workspaceId });
}

export async function listWorkspaces(
	projectId: string,
	cardId: string
): Promise<AgentWorkspace[]> {
	return tauriInvoke<AgentWorkspace[]>('list_workspaces', { projectId, cardId });
}

export async function getWorkspace(
	projectId: string,
	workspaceId: string
): Promise<AgentWorkspace> {
	return tauriInvoke<AgentWorkspace>('get_workspace', { projectId, workspaceId });
}
