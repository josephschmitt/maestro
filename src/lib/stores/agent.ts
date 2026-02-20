import { writable, derived, get } from 'svelte/store';
import type { AgentWorkspace } from '$lib/types/index.js';
import {
	launchAgent as launchAgentService,
	stopAgent as stopAgentService,
	listWorkspaces as listWorkspacesService,
	sendAgentInput as sendAgentInputService
} from '$lib/services/agent.js';
import { currentProject } from './project.js';

export interface AgentOutputLine {
	stream: 'stdout' | 'stderr';
	line: string;
}

export const workspaces = writable<AgentWorkspace[]>([]);
export const activeWorkspaceId = writable<string | null>(null);
export const agentOutput = writable<Map<string, AgentOutputLine[]>>(new Map());

const unlistenFns = new Map<string, () => void>();

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

export async function startAgent(cardId: string, statusGroup: string): Promise<AgentWorkspace> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	const workspace = await launchAgentService(project.id, cardId, statusGroup);

	await loadWorkspaces(cardId);
	activeWorkspaceId.set(workspace.id);

	await listenForOutput(workspace.id);

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

export async function sendInput(workspaceId: string, text: string): Promise<void> {
	await sendAgentInputService(workspaceId, text);
}

async function listenForOutput(workspaceId: string): Promise<void> {
	try {
		const { listen } = await import('@tauri-apps/api/event');

		const unlistenOutput = await listen<AgentOutputLine>(
			`agent-output-${workspaceId}`,
			(event) => {
				agentOutput.update((m) => {
					const lines = m.get(workspaceId) ?? [];
					lines.push(event.payload);
					m.set(workspaceId, [...lines]);
					return new Map(m);
				});
			}
		);

		const unlistenExit = await listen<{ workspace_id: string; exit_code: number | null; status: string }>(
			`agent-exit-${workspaceId}`,
			(event) => {
				workspaces.update((ws) =>
					ws.map((w) =>
						w.id === workspaceId ? { ...w, status: event.payload.status as AgentWorkspace['status'] } : w
					)
				);
				cleanupListener(workspaceId);
			}
		);

		const cleanup = () => {
			unlistenOutput();
			unlistenExit();
		};
		unlistenFns.set(workspaceId, cleanup);
	} catch {
		// Not in Tauri environment (browser mock mode) — skip event listening
	}
}

function cleanupListener(workspaceId: string): void {
	const unlisten = unlistenFns.get(workspaceId);
	if (unlisten) {
		unlisten();
		unlistenFns.delete(workspaceId);
	}
}

export function cleanupAllListeners(): void {
	for (const [id, unlisten] of unlistenFns) {
		unlisten();
	}
	unlistenFns.clear();
}
