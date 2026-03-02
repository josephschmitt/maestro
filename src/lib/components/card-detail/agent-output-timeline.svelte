<script lang="ts">
	import type { TimelineEntry } from '$lib/types/index.js';
	import { parseAnsi } from '$lib/utils/ansi-parser.js';
	import InlineToolCard from './inline-tool-card.svelte';
	import ChevronsDownUpIcon from '@lucide/svelte/icons/chevrons-down-up';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';

	let {
		entries,
		streaming = false
	}: {
		entries: TimelineEntry[];
		streaming?: boolean;
	} = $props();

	let timelineEl: HTMLDivElement | undefined = $state();
	let userScrolled = $state(false);
	let allExpanded = $state(false);
	let expandKey = $state(0);

	const hasTools = $derived(entries.some((e) => e.type === 'tool'));

	$effect(() => {
		void entries;
		if (timelineEl && !userScrolled && streaming) {
			timelineEl.scrollTop = timelineEl.scrollHeight;
		}
	});

	function handleScroll() {
		if (!timelineEl) return;
		const isAtBottom = timelineEl.scrollHeight - timelineEl.scrollTop - timelineEl.clientHeight < 40;
		userScrolled = !isAtBottom;
	}

	function collapseAll() {
		allExpanded = false;
		expandKey++;
	}

	function expandAll() {
		allExpanded = true;
		expandKey++;
	}

	function renderLine(line: string): string {
		return parseAnsi(line);
	}
</script>

<div class="flex h-full flex-col">
	{#if hasTools}
		<div class="flex items-center gap-1 border-b border-border/50 px-3 py-1.5">
			<button
				class="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
				onclick={expandAll}
				title="Expand all"
			>
				<ChevronsUpDownIcon class="size-3" />
				Expand
			</button>
			<button
				class="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
				onclick={collapseAll}
				title="Collapse all"
			>
				<ChevronsDownUpIcon class="size-3" />
				Collapse
			</button>
		</div>
	{/if}

	<div
		bind:this={timelineEl}
		class="min-h-0 flex-1 overflow-y-auto p-3"
		onscroll={handleScroll}
		role="log"
		aria-label="Agent output timeline"
		tabindex={-1}
	>
		<div class="flex flex-col gap-1.5">
			{#each entries as entry, i (entry.type === 'tool' ? entry.invocation.id : `text-${i}`)}
				{#if entry.type === 'tool'}
					{#key expandKey}
						<InlineToolCard invocation={entry.invocation} expanded={allExpanded} />
					{/key}
				{:else}
					<div class="font-mono text-xs leading-relaxed text-green-400">
						{#each entry.lines as line, j (j)}
							<div class={line.stream === 'stderr' ? 'text-red-400' : ''}>
								{@html renderLine(line.line)}
							</div>
						{/each}
					</div>
				{/if}
			{/each}
		</div>
	</div>
</div>
