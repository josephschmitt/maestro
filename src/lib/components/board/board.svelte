<script lang="ts">
	import { STATUS_GROUPS } from '$lib/types/index.js';
	import type { CardProgress } from '$lib/types/index.js';
	import { cards, cardsByStatus, addCard, getCardProgress } from '$lib/stores/cards.js';
	import { statuses as allStatuses } from '$lib/stores/statuses.js';
	import { statusesByGroup } from '$lib/stores/statuses.js';
	import StatusGroupColumn from './status-group-column.svelte';
	import EmptyState from './empty-state.svelte';

	let hasStatuses = $derived($allStatuses.length > 0);
	let totalCards = $derived($cards.filter((c) => c.parent_id === null).length);

	function getProgress(cardId: string): CardProgress | null {
		const progress = getCardProgress(cardId);
		if (progress.total === 0) return null;
		return progress;
	}

	async function handleAddCard(statusId: string, title: string) {
		await addCard(title, { statusId });
	}

	function getOriginalStatusId(cardId: string): string | undefined {
		return $cards.find((c) => c.id === cardId)?.status_id;
	}
</script>

{#if !hasStatuses}
	<EmptyState />
{:else}
	<div class="flex flex-1 flex-col overflow-hidden">
		{#if totalCards === 0}
			<div class="flex items-center justify-center border-b border-border px-4 py-3">
				<p class="text-sm text-muted-foreground">
					No cards yet. Use the + button in any column to add your first card.
				</p>
			</div>
		{/if}
		<div class="flex flex-1 gap-3 overflow-x-auto p-4">
			{#each STATUS_GROUPS as group (group)}
				<StatusGroupColumn
					{group}
					statuses={$statusesByGroup.get(group) ?? []}
					cardsByStatus={$cardsByStatus}
					{getProgress}
					onAddCard={handleAddCard}
					{getOriginalStatusId}
				/>
			{/each}
		</div>
	</div>
{/if}
