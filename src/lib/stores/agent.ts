import { writable, derived, get } from 'svelte/store';
import type { AgentWorkspace } from '$lib/types/index.js';
import {
	launchAgent as launchAgentService,
	stopAgent as stopAgentService,
	resumeAgent as resumeAgentService,
	listWorkspaces as listWorkspacesService,
	sendAgentInput as sendAgentInputService
} from '$lib/services/agent.js';
import { connectAgentStream, type AgentStreamConnection } from '$lib/services/agent-ws.js';
import { currentProject } from './project.js';

export interface AgentOutputLine {
	stream: 'stdout' | 'stderr';
	line: string;
}

export const workspaces = writable<AgentWorkspace[]>([]);
export const activeWorkspaceId = writable<string | null>(null);
export const agentOutput = writable<Map<string, AgentOutputLine[]>>(new Map());

const agentConnections = new Map<string, AgentStreamConnection>();

export const activeWorkspace = derived(
	[workspaces, activeWorkspaceId],
	([$workspaces, $activeWorkspaceId]) =>
		$workspaces.find((w) => w.id === $activeWorkspaceId) ?? null
);

export const activeOutput = derived(
	[agentOutput, activeWorkspaceId],
	([$agentOutput, $activeWorkspaceId]) => {
		if (!$activeWorkspaceId) return [];
		return $agentOutput.get($activeWorkspaceId) ?? [];
	}
);

export async function loadWorkspaces(cardId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	const list = await listWorkspacesService(project.id, cardId);
	workspaces.set(list);
}

export async function startAgent(
	cardId: string,
	statusId: string,
	worktreePath?: string | null,
	branchName?: string | null,
	repoPath?: string | null
): Promise<AgentWorkspace> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	const workspace = await launchAgentService(
		project.id,
		cardId,
		statusId,
		worktreePath,
		branchName,
		repoPath
	);

	await loadWorkspaces(cardId);
	activeWorkspaceId.set(workspace.id);

	listenForOutput(workspace.id);

	return workspace;
}

export async function stopCurrentAgent(workspaceId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	await stopAgentService(project.id, workspaceId);

	const ws = get(workspaces);
	const current = ws.find((w) => w.id === workspaceId);
	if (current) {
		await loadWorkspaces(current.card_id);
	}

	cleanupListener(workspaceId);
}

export async function resumeAgent(workspaceId: string, cardId: string): Promise<AgentWorkspace> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	const workspace = await resumeAgentService(project.id, workspaceId, cardId);

	await loadWorkspaces(cardId);
	activeWorkspaceId.set(workspace.id);

	listenForOutput(workspace.id);

	return workspace;
}

export async function sendInput(workspaceId: string, text: string): Promise<void> {
	const connection = agentConnections.get(workspaceId);
	if (connection) {
		connection.sendInput(text);
	} else {
		await sendAgentInputService(workspaceId, text);
	}
}

function listenForOutput(workspaceId: string): void {
	if (agentConnections.has(workspaceId)) {
		return;
	}

	const connection = connectAgentStream(
		workspaceId,
		(event) => {
			agentOutput.update((m) => {
				const lines = m.get(workspaceId) ?? [];
				lines.push(event);
				m.set(workspaceId, [...lines]);
				return new Map(m);
			});
		},
		(event) => {
			workspaces.update((ws) =>
				ws.map((w) =>
					w.id === workspaceId ? { ...w, status: event.status as AgentWorkspace['status'] } : w
				)
			);
			cleanupListener(workspaceId);
		}
	);

	agentConnections.set(workspaceId, connection);
}

function cleanupListener(workspaceId: string): void {
	const connection = agentConnections.get(workspaceId);
	if (connection) {
		connection.disconnect();
		agentConnections.delete(workspaceId);
	}
}

export function cleanupAllListeners(): void {
	for (const [, connection] of agentConnections) {
		connection.disconnect();
	}
	agentConnections.clear();
}
