<script lang="ts">
	import {
		workspaces,
		activeWorkspaceId,
		activeWorkspace,
		activeOutput,
		loadWorkspaces,
		startAgent,
		stopCurrentAgent,
		sendInput,
		resumeAgent
	} from '$lib/stores/agent.js';
	import AgentTerminal from '../agent-terminal.svelte';
	import AgentControls from '../agent-controls.svelte';
	import CircleDotIcon from '@lucide/svelte/icons/circle-dot';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';

	let {
		cardId,
		statusGroup
	}: {
		cardId: string;
		statusGroup: string;
	} = $props();

	$effect(() => {
		activeWorkspaceId.set(null);
		loadWorkspaces(cardId);
	});

	async function handleStart() {
		await startAgent(cardId, statusGroup);
	}

	async function handleStop() {
		const ws = $activeWorkspace;
		if (!ws) return;
		await stopCurrentAgent(ws.id);
	}

	function handleSend(text: string) {
		const ws = $activeWorkspace;
		if (!ws) return;
		sendInput(ws.id, text);
	}

	async function handleResume(workspaceId: string) {
		await resumeAgent(workspaceId, cardId);
	}

	function handleSelectWorkspace(id: string) {
		activeWorkspaceId.set(id);
	}

	function statusBadgeClass(status: string): string {
		switch (status) {
			case 'running':
				return 'bg-green-500/20 text-green-400';
			case 'completed':
				return 'bg-blue-500/20 text-blue-400';
			case 'failed':
				return 'bg-red-500/20 text-red-400';
			case 'paused':
				return 'bg-yellow-500/20 text-yellow-400';
			default:
				return 'bg-muted text-muted-foreground';
		}
	}
</script>

{#if $activeWorkspace}
	<div class="flex h-[400px] flex-col gap-3">
		<div class="flex items-center justify-between">
			<button
				class="text-sm text-muted-foreground hover:text-foreground"
				onclick={() => activeWorkspaceId.set(null)}
			>
				&larr; All Sessions
			</button>
			<span class={`rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass($activeWorkspace.status)}`}>
				{$activeWorkspace.status}
			</span>
		</div>

		<div class="min-h-0 flex-1">
			<AgentTerminal lines={$activeOutput} />
		</div>

		{#if $activeWorkspace.status === 'failed' && $activeWorkspace.session_id}
			<button
				class="inline-flex w-full items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				onclick={() => $activeWorkspace && handleResume($activeWorkspace.id)}
				aria-label="Resume agent"
			>
				<RotateCcwIcon size={14} class="mr-1" />
				Resume Session
			</button>
		{:else}
			<AgentControls
				workspace={$activeWorkspace}
				onstart={handleStart}
				onstop={handleStop}
				onsend={handleSend}
			/>
		{/if}
	</div>
{:else}
	<div class="flex flex-col gap-3">
		<AgentControls
			workspace={null}
			onstart={handleStart}
			onstop={handleStop}
			onsend={handleSend}
		/>

		{#if $workspaces.length > 0}
			<div class="flex flex-col gap-1">
				<h4 class="text-xs font-medium text-muted-foreground uppercase">Sessions</h4>
				{#each $workspaces as ws (ws.id)}
					<button
						class="flex items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
						onclick={() => handleSelectWorkspace(ws.id)}
					>
						<CircleDotIcon size={12} class={ws.status === 'running' ? 'text-green-400' : 'text-muted-foreground'} />
						<span class="flex-1 truncate">{ws.agent_type}</span>
						<span class={`rounded-full px-2 py-0.5 text-xs ${statusBadgeClass(ws.status)}`}>
							{ws.status}
						</span>
						{#if ws.status === 'failed' && ws.session_id}
							<button
								class="rounded px-1.5 py-0.5 text-xs text-primary hover:bg-primary/10"
								onclick={(e) => { e.stopPropagation(); handleResume(ws.id); }}
								aria-label="Resume session"
							>
								Resume
							</button>
						{/if}
						<span class="text-xs text-muted-foreground">
							{new Date(ws.attached_at).toLocaleString()}
						</span>
					</button>
				{/each}
			</div>
		{:else}
			<p class="py-4 text-center text-sm text-muted-foreground">
				No agent sessions yet. Start one to run an agent on this card.
			</p>
		{/if}
	</div>
{/if}
