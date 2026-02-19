<script lang="ts">
	import type { Status, CardWithStatus, CardProgress } from '$lib/types/index.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import CardItem from './card-item.svelte';
	import AddCardInline from './add-card-inline.svelte';

	let {
		status,
		cards,
		getProgress,
		onAddCard
	}: {
		status: Status;
		cards: CardWithStatus[];
		getProgress: (cardId: string) => CardProgress | null;
		onAddCard: (statusId: string, title: string) => void;
	} = $props();
</script>

<div class="flex min-h-0 w-64 shrink-0 flex-col">
	<div class="flex items-center gap-2 px-2 py-2">
		<StatusBadge name={status.name} group={status.group} />
		<span class="text-xs text-muted-foreground">{cards.length}</span>
	</div>

	<div class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto px-1 pb-1">
		{#each cards as card (card.id)}
			<CardItem {card} progress={getProgress(card.id)} />
		{/each}
	</div>

	<div class="mt-1 px-1 pb-1">
		<AddCardInline onAdd={(title) => onAddCard(status.id, title)} />
	</div>
</div>
