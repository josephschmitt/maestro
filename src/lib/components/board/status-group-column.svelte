<script lang="ts">
	import type { Status, StatusGroup, CardWithStatus, CardProgress } from '$lib/types/index.js';
	import StatusColumn from './status-column.svelte';

	let {
		group,
		statuses,
		cardsByStatus,
		getProgress,
		onAddCard,
		getOriginalStatusId
	}: {
		group: StatusGroup;
		statuses: Status[];
		cardsByStatus: Map<string, CardWithStatus[]>;
		getProgress: (cardId: string) => CardProgress | null;
		onAddCard: (statusId: string, title: string) => void;
		getOriginalStatusId: (cardId: string) => string | undefined;
	} = $props();

	const groupLabels: Record<StatusGroup, string> = {
		Backlog: 'Backlog',
		Unstarted: 'Unstarted',
		Started: 'Started',
		Completed: 'Completed',
		Cancelled: 'Cancelled'
	};

	const groupBorderColors: Record<StatusGroup, string> = {
		Backlog: 'border-gray-200 dark:border-gray-700',
		Unstarted: 'border-blue-200 dark:border-blue-800',
		Started: 'border-amber-200 dark:border-amber-800',
		Completed: 'border-green-200 dark:border-green-800',
		Cancelled: 'border-red-200 dark:border-red-800'
	};
</script>

{#if statuses.length > 0}
	<div class="flex shrink-0 gap-0.5">
		{#if statuses.length > 1}
			<div class="flex flex-col items-center px-1 pt-1">
				<span class="text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60 [writing-mode:vertical-lr] rotate-180">
					{groupLabels[group]}
				</span>
			</div>
		{/if}
		<div class="flex gap-1 rounded-lg border {groupBorderColors[group]} bg-muted/30 p-1.5">
			{#each statuses as status (status.id)}
				<StatusColumn
					{status}
					cards={cardsByStatus.get(status.id) ?? []}
					{getProgress}
					{onAddCard}
					{getOriginalStatusId}
				/>
			{/each}
		</div>
	</div>
{/if}
