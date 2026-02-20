<script lang="ts">
	import type { AgentOutputLine } from '$lib/stores/agent.js';

	let {
		lines
	}: {
		lines: AgentOutputLine[];
	} = $props();

	let terminalEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		lines;
		if (terminalEl) {
			terminalEl.scrollTop = terminalEl.scrollHeight;
		}
	});
</script>

<div
	bind:this={terminalEl}
	class="h-full overflow-y-auto rounded-md border border-border bg-black p-3 font-mono text-xs leading-relaxed text-green-400"
	role="log"
	aria-label="Agent output"
	tabindex={-1}
>
	{#if lines.length === 0}
		<span class="text-muted-foreground">Waiting for output...</span>
	{:else}
		{#each lines as line, i (i)}
			<div class={line.stream === 'stderr' ? 'text-red-400' : ''}>
				{line.line}
			</div>
		{/each}
	{/if}
</div>
