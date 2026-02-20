<script lang="ts">
	import type { AgentWorkspace } from '$lib/types/index.js';
	import PlayIcon from '@lucide/svelte/icons/play';
	import SquareIcon from '@lucide/svelte/icons/square';
	import SendIcon from '@lucide/svelte/icons/send-horizontal';

	let {
		workspace,
		onstart,
		onstop,
		onsend
	}: {
		workspace: AgentWorkspace | null;
		onstart: () => void;
		onstop: () => void;
		onsend: (text: string) => void;
	} = $props();

	let inputText = $state('');
	let isRunning = $derived(workspace?.status === 'running');

	function handleSend() {
		const text = inputText.trim();
		if (!text) return;
		onsend(text);
		inputText = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<div class="flex flex-col gap-2">
	{#if !workspace || !isRunning}
		<button
			class="inline-flex w-full items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			onclick={onstart}
			aria-label="Start agent"
		>
			<PlayIcon size={14} class="mr-1" />
			Start Agent
		</button>
	{:else}
		<div class="flex gap-2">
			<input
				type="text"
				bind:value={inputText}
				onkeydown={handleKeydown}
				placeholder="Send message to agent..."
				class="flex-1 rounded-md border border-border bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				aria-label="Agent input"
			/>
			<button
				class="inline-flex items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				onclick={handleSend}
				disabled={!inputText.trim()}
				aria-label="Send input"
			>
				<SendIcon size={14} />
			</button>
			<button
				class="inline-flex items-center justify-center rounded-md bg-destructive px-3 py-1.5 text-sm font-medium text-destructive-foreground hover:bg-destructive/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				onclick={onstop}
				aria-label="Stop agent"
			>
				<SquareIcon size={14} />
			</button>
		</div>
	{/if}
</div>
