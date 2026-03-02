import type { AgentWorkspace } from '$lib/types/index.js';
import { agentOutput, toolInvocations } from '$lib/stores/agent.js';
import type { AgentOutputLine, ToolInvocation } from '$lib/types/index.js';
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

export function send_agent_input(_: Record<string, unknown>): void {
	void _; // No-op in mock mode
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
		session_id: oldWs?.session_id ?? null,
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

export function archive_card_workspaces(args: Record<string, unknown>): void {
	const store = getStore();
	const cardId = args.cardId as string;
	for (const ws of store.agentWorkspaces) {
		if (ws.card_id === cardId && ws.status !== 'completed' && ws.status !== 'failed') {
			ws.status = 'completed';
			ws.completed_at = nowISO();
		}
	}
}

type MockToolStep =
	| { type: 'text'; line: string }
	| { type: 'tool_start'; id: string; tool_name: string; input_summary: string }
	| { type: 'tool_end'; id: string; output_preview?: string; output_full?: string; error?: string };

function simulateMockOutput(workspaceId: string): void {
	const store = getStore();

	const steps: MockToolStep[] = [
		{ type: 'text', line: 'Resolving agent configuration...' },
		{ type: 'text', line: 'Starting exploration session...' },
		{
			type: 'tool_start',
			id: `tool-${newId()}`,
			tool_name: 'Read',
			input_summary: 'src/lib/components/board/board-column.svelte'
		},
		{
			type: 'text',
			line: 'Reading card context and artifacts...'
		},
		{
			type: 'tool_end',
			id: '', // will be filled
			output_preview: '<script lang="ts">\n  import { Card } from \'$lib/types\';\n  // Board column component\n</script>',
			output_full: '<script lang="ts">\n  import { Card } from \'$lib/types\';\n  // Board column component with full implementation\n  export let cards: Card[] = [];\n</script>\n\n<div class="column">\n  {#each cards as card}\n    <slot {card} />\n  {/each}\n</div>'
		},
		{
			type: 'tool_start',
			id: `tool-${newId()}`,
			tool_name: 'Grep',
			input_summary: 'import.*AgentWorkspace'
		},
		{
			type: 'tool_end',
			id: '',
			output_preview: 'src/lib/types/index.ts:74: export interface AgentWorkspace {\nsrc/lib/stores/agent.ts:2: import type { AgentWorkspace } from...'
		},
		{
			type: 'tool_start',
			id: `tool-${newId()}`,
			tool_name: 'Bash',
			input_summary: 'npm run check'
		},
		{
			type: 'tool_end',
			id: '',
			output_preview: '$ npm run check\n\n> maestro@0.1.0 check\n> svelte-check\n\n0 errors, 0 warnings'
		},
		{
			type: 'tool_start',
			id: `tool-${newId()}`,
			tool_name: 'Edit',
			input_summary: 'src/lib/stores/agent.ts'
		},
		{
			type: 'tool_end',
			id: '',
			output_preview: 'Updated 3 lines in agent.ts',
			error: undefined
		},
		{
			type: 'tool_start',
			id: `tool-${newId()}`,
			tool_name: 'Write',
			input_summary: 'src/lib/components/card-detail/new-component.svelte'
		},
		{
			type: 'tool_end',
			id: '',
			error: 'EPERM: permission denied, open \'src/lib/components/card-detail/new-component.svelte\''
		},
		{ type: 'text', line: 'Agent ready. Task complete.' }
	];

	// Link tool_end entries to their corresponding tool_start entries
	let lastToolStartId = '';
	for (const step of steps) {
		if (step.type === 'tool_start') {
			lastToolStartId = step.id;
		} else if (step.type === 'tool_end') {
			step.id = lastToolStartId;
		}
	}

	let i = 0;
	const interval = setInterval(() => {
		if (i >= steps.length) {
			clearInterval(interval);
			const ws = store.agentWorkspaces.find((w) => w.id === workspaceId);
			if (ws && ws.status === 'running') {
				ws.status = 'completed';
				ws.completed_at = nowISO();
			}
			return;
		}

		const step = steps[i];

		if (step.type === 'text') {
			const outputLine: AgentOutputLine = { stream: 'stdout', line: step.line };
			agentOutput.update((m) => {
				const lines = m.get(workspaceId) ?? [];
				lines.push(outputLine);
				m.set(workspaceId, [...lines]);
				return new Map(m);
			});
		} else if (step.type === 'tool_start') {
			const toolLine = JSON.stringify({
				maestro_tool: 'start',
				id: step.id,
				tool_name: step.tool_name,
				input_summary: step.input_summary
			});
			const outputLine: AgentOutputLine = { stream: 'stdout', line: toolLine };
			agentOutput.update((m) => {
				const lines = m.get(workspaceId) ?? [];
				lines.push(outputLine);
				m.set(workspaceId, [...lines]);
				return new Map(m);
			});

			const invocation: ToolInvocation = {
				id: step.id,
				tool_name: step.tool_name,
				status: 'running',
				started_at: nowISO(),
				input_summary: step.input_summary
			};
			toolInvocations.update((m) => {
				const wsTools = m.get(workspaceId) ?? new Map<string, ToolInvocation>();
				wsTools.set(step.id, invocation);
				m.set(workspaceId, new Map(wsTools));
				return new Map(m);
			});
		} else if (step.type === 'tool_end') {
			const toolLine = JSON.stringify({
				maestro_tool: 'end',
				id: step.id,
				output_preview: step.output_preview,
				output_full: step.output_full,
				error: step.error
			});
			const outputLine: AgentOutputLine = { stream: 'stdout', line: toolLine };
			agentOutput.update((m) => {
				const lines = m.get(workspaceId) ?? [];
				lines.push(outputLine);
				m.set(workspaceId, [...lines]);
				return new Map(m);
			});

			toolInvocations.update((m) => {
				const wsTools = m.get(workspaceId) ?? new Map<string, ToolInvocation>();
				const existing = wsTools.get(step.id);
				if (existing) {
					const completedAt = nowISO();
					const durationMs = new Date(completedAt).getTime() - new Date(existing.started_at).getTime();
					wsTools.set(step.id, {
						...existing,
						status: step.error ? 'failed' : 'completed',
						completed_at: completedAt,
						duration_ms: durationMs,
						output_preview: step.output_preview,
						output_full: step.output_full,
						error: step.error
					});
				}
				m.set(workspaceId, new Map(wsTools));
				return new Map(m);
			});
		}

		i++;
	}, 400);
}
