<script lang="ts">
	import Skeleton from './skeleton.svelte';
	import SkeletonCard from './skeleton-card.svelte';

	let {
		cardCount = 3
	}: {
		cardCount?: number;
	} = $props();
</script>

<div class="flex w-64 shrink-0 flex-col rounded-lg border border-border bg-muted/30 p-3">
	<div class="mb-3 flex items-center justify-between">
		<Skeleton class="h-5 w-24" />
		<Skeleton class="size-6 rounded" />
	</div>
	<div class="flex flex-col gap-2">
		{#each Array(cardCount) as _, i (i)}
			<SkeletonCard />
		{/each}
	</div>
</div>
