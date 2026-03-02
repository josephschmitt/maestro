<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import FileTextIcon from '@lucide/svelte/icons/file-text';

	let {
		preview,
		charCount,
		filename,
		onremove,
		onclick
	}: {
		preview: string;
		charCount: number;
		filename?: string;
		onremove: () => void;
		onclick: () => void;
	} = $props();

	function handleRemove(e: MouseEvent) {
		e.stopPropagation();
		onremove();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Delete' || e.key === 'Backspace') {
			e.preventDefault();
			onremove();
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class="group flex max-w-64 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-muted/50 px-2 py-1 text-left text-xs transition-colors hover:bg-muted"
	onclick={onclick}
	onkeydown={handleKeydown}
	role="button"
	tabindex="0"
	title="Click to expand"
>
	<FileTextIcon size={12} class="shrink-0 text-muted-foreground" />
	<span class="truncate text-foreground">
		{#if filename}
			{filename}
		{:else}
			{preview}
		{/if}
	</span>
	<span class="shrink-0 text-muted-foreground">
		{charCount.toLocaleString()} chars
	</span>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<span
		class="shrink-0 cursor-pointer rounded p-0.5 text-muted-foreground opacity-0 transition-opacity hover:bg-background hover:text-foreground group-hover:opacity-100"
		onclick={handleRemove}
		role="button"
		tabindex="0"
		aria-label="Remove attachment"
	>
		<XIcon size={12} />
	</span>
</div>
