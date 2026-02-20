<script lang="ts">
	import Board from '$lib/components/board/board.svelte';
	import CardDetailPanel from '$lib/components/card-detail/card-detail-panel.svelte';
	import { currentProject } from '$lib/stores/project.js';
	import { loadStatuses } from '$lib/stores/statuses.js';
	import { loadCards, cards } from '$lib/stores/cards.js';
	import { loadLinkedDirectories } from '$lib/stores/directories.js';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';

	let loaded = $state(false);

	let selectedCardId = $derived($page.url.searchParams.get('card'));

	function openCard(cardId: string) {
		const url = new URL($page.url);
		url.searchParams.set('card', cardId);
		goto(url.toString(), { replaceState: false, noScroll: true });
	}

	function closeCard() {
		const url = new URL($page.url);
		url.searchParams.delete('card');
		goto(url.toString(), { replaceState: false, noScroll: true });
	}

	onMount(() => {
		return currentProject.subscribe(async (project) => {
			if (project) {
				await Promise.all([loadStatuses(), loadCards(), loadLinkedDirectories()]);
			}
			loaded = true;
		});
	});
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex h-14 items-center border-b border-border px-6">
		<h1 class="text-lg font-semibold">
			{#if $currentProject}
				{$currentProject.name} — Board
			{:else}
				Board
			{/if}
		</h1>
	</div>

	<div class="flex flex-1 overflow-hidden">
		{#if !$currentProject}
			<div class="flex flex-1 items-center justify-center">
				<p class="text-sm text-muted-foreground">Select a project to view the board.</p>
			</div>
		{:else if !loaded}
			<div class="flex flex-1 items-center justify-center">
				<p class="text-sm text-muted-foreground">Loading...</p>
			</div>
		{:else}
			<Board onCardClick={openCard} />
		{/if}
	</div>
</div>

{#if selectedCardId}
	<CardDetailPanel
		cardId={selectedCardId}
		cards={$cards}
		onclose={closeCard}
		onnavigate={openCard}
	/>
{/if}
