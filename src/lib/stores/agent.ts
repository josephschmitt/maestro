import { writable, derived, get } from 'svelte/store';
import type {
	AgentWorkspace,
	AgentOutputLine,
	ToolInvocation,
	TimelineEntry
} from '$lib/types/index.js';
import {
	launchAgent as launchAgentService,
	stopAgent as stopAgentService,
	resumeAgent as resumeAgentService,
	listWorkspaces as listWorkspacesService,
	sendAgentInput as sendAgentInputService
} from '$lib/services/agent.js';
import { connectAgentStream, type AgentStreamConnection } from '$lib/services/agent-ws.js';
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { parseToolEvent, type ToolEvent } from '$lib/utils/tool-parser.js';

export type { AgentOutputLine };

export const workspaces = writable<AgentWorkspace[]>([]);
export const activeWorkspaceId = writable<string | null>(null);
export const agentOutput = writable<Map<string, AgentOutputLine[]>>(new Map());
export const toolInvocations = writable<Map<string, Map<string, ToolInvocation>>>(new Map());

const agentConnections = new Map<string, AgentStreamConnection>();

let currentCardId: string | null = null;

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

export const activeToolInvocations = derived(
	[toolInvocations, activeWorkspaceId],
	([$toolInvocations, $activeWorkspaceId]) => {
		if (!$activeWorkspaceId) return new Map<string, ToolInvocation>();
		return $toolInvocations.get($activeWorkspaceId) ?? new Map<string, ToolInvocation>();
	}
);

export const activeTimeline = derived(
	[activeOutput, activeToolInvocations],
	([$activeOutput, $activeToolInvocations]) => {
		return buildTimeline($activeOutput, $activeToolInvocations);
	}
);

export function buildTimeline(
	lines: AgentOutputLine[],
	tools: Map<string, ToolInvocation>
): TimelineEntry[] {
	if (lines.length === 0 && tools.size === 0) return [];

	const entries: TimelineEntry[] = [];
	let currentTextBlock: AgentOutputLine[] = [];

	for (const line of lines) {
		const event = parseToolEvent(line.line);
		if (event) {
			const tool = tools.get(event.id);
			if (tool) {
				if (currentTextBlock.length > 0) {
					entries.push({ type: 'text', lines: [...currentTextBlock] });
					currentTextBlock = [];
				}
				if (event.type === 'tool_start' || !entries.some(e => e.type === 'tool' && e.invocation.id === tool.id)) {
					entries.push({ type: 'tool', invocation: tool });
				}
			}
		} else {
			currentTextBlock.push(line);
		}
	}

	if (currentTextBlock.length > 0) {
		entries.push({ type: 'text', lines: currentTextBlock });
	}

	return entries;
}

function handleToolEvent(workspaceId: string, event: ToolEvent): void {
	toolInvocations.update((m) => {
		const wsTools = m.get(workspaceId) ?? new Map<string, ToolInvocation>();

		if (event.type === 'tool_start') {
			wsTools.set(event.id, {
				id: event.id,
				tool_name: event.tool_name,
				status: 'running',
				started_at: new Date().toISOString(),
				input_summary: event.input_summary
			});
		} else if (event.type === 'tool_end') {
			const existing = wsTools.get(event.id);
			if (existing) {
				const completedAt = new Date().toISOString();
				const durationMs = new Date(completedAt).getTime() - new Date(existing.started_at).getTime();
				wsTools.set(event.id, {
					...existing,
					status: event.error ? 'failed' : 'completed',
					completed_at: completedAt,
					duration_ms: durationMs,
					output_preview: event.output_preview,
					output_full: event.output_full,
					error: event.error
				});
			}
		}

		m.set(workspaceId, new Map(wsTools));
		return new Map(m);
	});
}

export async function loadWorkspaces(cardId: string): Promise<void> {
	currentCardId = cardId;
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
			const toolEvent = parseToolEvent(event.line);
			if (toolEvent) {
				handleToolEvent(workspaceId, toolEvent);
			}

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

listenEvent<{ project_id: string }>('workspaces-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id && currentCardId) {
		loadWorkspaces(currentCardId);
	}
});
