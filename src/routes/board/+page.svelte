<script lang="ts">
	import Board from '$lib/components/board/board.svelte';
	import { currentProject } from '$lib/stores/project.js';
	import { loadStatuses } from '$lib/stores/statuses.js';
	import { loadCards } from '$lib/stores/cards.js';
	import { onMount } from 'svelte';

	let loaded = $state(false);

	onMount(() => {
		return currentProject.subscribe(async (project) => {
			if (project) {
				await Promise.all([loadStatuses(), loadCards()]);
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
			<Board />
		{/if}
	</div>
</div>
