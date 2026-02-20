<script lang="ts">
	import type { CardWithStatus, CardProgress } from '$lib/types/index.js';
	import { SHADOW_ITEM_MARKER_PROPERTY_NAME } from 'svelte-dnd-action';
	import { dragHandle } from 'svelte-dnd-action';
	import GripVerticalIcon from '@lucide/svelte/icons/grip-vertical';

	let {
		card,
		progress,
		onclick
	}: {
		card: CardWithStatus & { isDndShadowItem?: boolean };
		progress: CardProgress | null;
		onclick?: (cardId: string) => void;
	} = $props();

	function labelColor(label: string): string {
		let hash = 0;
		for (let i = 0; i < label.length; i++) {
			hash = label.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = Math.abs(hash) % 360;
		return `hsl(${hue}, 60%, 85%)`;
	}

	function labelTextColor(label: string): string {
		let hash = 0;
		for (let i = 0; i < label.length; i++) {
			hash = label.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = Math.abs(hash) % 360;
		return `hsl(${hue}, 50%, 30%)`;
	}

	let isShadow = $derived(card[SHADOW_ITEM_MARKER_PROPERTY_NAME] === true);
	let visibleLabels = $derived(card.labels.slice(0, 3));
	let extraLabelCount = $derived(Math.max(0, card.labels.length - 3));
	let progressPercent = $derived(
		progress && progress.total > 0
			? Math.round((progress.completed / progress.total) * 100)
			: null
	);
</script>

<div
	class="group flex rounded-lg border bg-card shadow-sm transition-colors
		{isShadow
			? 'border-dashed border-primary/50 bg-primary/5 opacity-50'
			: 'border-border hover:border-ring/50 hover:shadow-md'}"
	role="button"
	tabindex="0"
	onclick={() => onclick?.(card.id)}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onclick?.(card.id); } }}
>
	<div
		class="flex shrink-0 cursor-grab items-center px-1 text-muted-foreground/40 hover:text-muted-foreground active:cursor-grabbing"
		use:dragHandle
		aria-label="Drag to reorder"
	>
		<GripVerticalIcon size={14} />
	</div>

	<div class="min-w-0 flex-1 p-3 pl-0">
		<p class="truncate text-sm font-medium text-card-foreground">{card.title}</p>

		{#if visibleLabels.length > 0}
			<div class="mt-1.5 flex flex-wrap gap-1">
				{#each visibleLabels as label (label)}
					<span
						class="inline-flex rounded-full px-1.5 py-0.5 text-[10px] font-medium leading-tight"
						style="background-color: {labelColor(label)}; color: {labelTextColor(label)}"
					>
						{label}
					</span>
				{/each}
				{#if extraLabelCount > 0}
					<span class="inline-flex rounded-full bg-muted px-1.5 py-0.5 text-[10px] font-medium leading-tight text-muted-foreground">
						+{extraLabelCount}
					</span>
				{/if}
			</div>
		{/if}

		<div class="mt-1.5 flex items-center gap-2">
			{#if progressPercent !== null}
				<div class="flex flex-1 items-center gap-1.5">
					<div class="h-1 flex-1 overflow-hidden rounded-full bg-muted">
						<div
							class="h-full rounded-full bg-green-500 transition-all"
							style="width: {progressPercent}%"
						></div>
					</div>
					<span class="text-[10px] text-muted-foreground">{progress?.completed}/{progress?.total}</span>
				</div>
			{/if}
			<span class="text-[10px] text-muted-foreground" title="Open questions (coming soon)">0 questions</span>
		</div>
	</div>
</div>
