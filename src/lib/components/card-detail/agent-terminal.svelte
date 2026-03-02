<script lang="ts">
	import type { AgentOutputLine, TimelineEntry } from '$lib/types/index.js';
	import AnimatedSpinner from '$lib/components/ui/animated-spinner.svelte';
	import AgentOutputTimeline from './agent-output-timeline.svelte';

	let {
		lines,
		timeline = [],
		streaming = false
	}: {
		lines: AgentOutputLine[];
		timeline?: TimelineEntry[];
		streaming?: boolean;
	} = $props();

	const hasTimeline = $derived(timeline.length > 0);
</script>

<div
	class="h-full overflow-hidden rounded-md border border-border bg-black"
	role="log"
	aria-label="Agent output"
	tabindex={-1}
>
	{#if lines.length === 0 && timeline.length === 0}
		<div class="flex h-full items-center justify-center p-3">
			<AnimatedSpinner context="coding" />
		</div>
	{:else if hasTimeline}
		<AgentOutputTimeline entries={timeline} {streaming} />
	{:else}
		<AgentOutputTimeline
			entries={[{ type: 'text', lines }]}
			{streaming}
		/>
	{/if}
</div>
