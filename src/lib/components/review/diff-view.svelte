<script lang="ts">
	import type { FileDiff } from '$lib/types/index.js';
	import DiffLineComponent from './diff-line.svelte';

	let { diff }: { diff: FileDiff } = $props();
</script>

<div class="flex flex-col gap-2">
	{#each diff.hunks as hunk, i (i)}
		<div class="rounded-md border border-border overflow-hidden">
			<div class="bg-muted px-3 py-1 text-xs text-muted-foreground font-mono">
				@@ -{hunk.old_start},{hunk.old_count} +{hunk.new_start},{hunk.new_count} @@
				{#if hunk.header}
					<span class="ml-1">{hunk.header}</span>
				{/if}
			</div>
			<div class="divide-y divide-border/30">
				{#each hunk.lines as line, j (j)}
					<DiffLineComponent {line} />
				{/each}
			</div>
		</div>
	{/each}

	{#if diff.hunks.length === 0}
		<p class="text-sm text-muted-foreground py-4 text-center">No changes in this file.</p>
	{/if}
</div>
