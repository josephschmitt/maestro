<script lang="ts">
	import type { Status, CardWithStatus, CardProgress } from '$lib/types/index.js';
	import { dragHandleZone, type DndEvent } from 'svelte-dnd-action';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import CardItem from './card-item.svelte';
	import AddCardInline from './add-card-inline.svelte';
	import { FLIP_DURATION_MS, DND_TYPE, handleFinalize } from '$lib/utils/dnd.js';
	import type { DndItem } from '$lib/utils/dnd.js';

	let {
		status,
		cards,
		getProgress,
		onAddCard,
		getOriginalStatusId,
		onCardClick
	}: {
		status: Status;
		cards: CardWithStatus[];
		getProgress: (cardId: string) => CardProgress | null;
		onAddCard: (statusId: string, title: string) => void;
		getOriginalStatusId: (cardId: string) => string | undefined;
		onCardClick?: (cardId: string) => void;
	} = $props();

	// eslint-disable-next-line svelte/prefer-writable-derived -- dndItems must be mutated by svelte-dnd-action on consider/finalize events
	let dndItems: DndItem[] = $state([]);

	$effect(() => {
		dndItems = [...cards];
	});

	function handleConsider(event: CustomEvent<{ items: DndItem[] }>) {
		dndItems = event.detail.items;
	}

	async function handleFinalizeEvent(event: CustomEvent<DndEvent<DndItem>>) {
		dndItems = event.detail.items;
		await handleFinalize(event, status.id, getOriginalStatusId);
	}
</script>

<div class="flex min-h-0 w-64 shrink-0 flex-col">
	<div class="flex items-center gap-2 px-2 py-2">
		<StatusBadge name={status.name} group={status.group} />
		<span class="text-xs text-muted-foreground">{cards.length}</span>
	</div>

	<div
		class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto px-1 pb-1"
		use:dragHandleZone={{
			items: dndItems,
			type: DND_TYPE,
			flipDurationMs: FLIP_DURATION_MS,
			dropTargetClasses: ['!border-primary/50', '!bg-primary/5'],
			centreDraggedOnCursor: true
		}}
		onconsider={handleConsider}
		onfinalize={handleFinalizeEvent}
	>
		{#each dndItems as card (card.id)}
			<CardItem {card} progress={getProgress(card.id)} onclick={onCardClick} />
		{/each}
	</div>

	<div class="mt-1 px-1 pb-1">
		<AddCardInline onAdd={(title) => onAddCard(status.id, title)} />
	</div>
</div>
