<script lang="ts">
	import type { CardWithStatus } from '$lib/types/index.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';

	let {
		subCards,
		onaddsubcard,
		onsubcardclick
	}: {
		subCards: CardWithStatus[];
		onaddsubcard: () => void;
		onsubcardclick: (cardId: string) => void;
	} = $props();
</script>

<div class="flex flex-col gap-2">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-medium text-foreground">Sub-cards</h3>
		<button
			class="inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
			onclick={onaddsubcard}
			aria-label="Add sub-card"
		>
			<PlusIcon size={12} />
			Add
		</button>
	</div>

	{#if subCards.length === 0}
		<p class="text-xs text-muted-foreground">No sub-cards yet.</p>
	{:else}
		<div class="flex flex-col gap-1">
			{#each subCards as subCard (subCard.id)}
				<button
					class="flex items-center justify-between rounded-md border border-border px-3 py-2 text-left text-sm hover:bg-muted/50"
					onclick={() => onsubcardclick(subCard.id)}
				>
					<span class="truncate">{subCard.title}</span>
					<StatusBadge name={subCard.status_name} group={subCard.status_group} />
				</button>
			{/each}
		</div>
	{/if}
</div>
