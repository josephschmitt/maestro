<script lang="ts">
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let {
		parentCardTitle,
		onboardclick,
		onparentclick
	}: {
		parentCardTitle: string;
		onboardclick: () => void;
		onparentclick?: () => void;
	} = $props();
</script>

<nav class="flex items-center gap-1 text-sm" aria-label="Breadcrumb">
	<button
		class="text-muted-foreground hover:text-foreground hover:underline"
		onclick={onboardclick}
	>
		Board
	</button>
	<ChevronRightIcon size={14} class="text-muted-foreground/60" />
	{#if onparentclick}
		<button
			class="truncate text-muted-foreground hover:text-foreground hover:underline"
			onclick={onparentclick}
		>
			{parentCardTitle}
		</button>
	{:else}
		<span class="truncate font-medium text-foreground">{parentCardTitle}</span>
	{/if}
</nav>
