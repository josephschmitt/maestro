<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { FocusRegionId } from '$lib/focus/types.js';
	import { pushRegion, popRegion } from '$lib/focus/context.js';
	import { onMount } from 'svelte';

	let {
		open,
		onclose,
		focusRegion = 'card-detail' as FocusRegionId,
		children
	}: {
		open: boolean;
		onclose: () => void;
		focusRegion?: FocusRegionId;
		children: Snippet;
	} = $props();

	let pushed = $state(false);

	$effect(() => {
		if (open && !pushed) {
			pushRegion(focusRegion);
			pushed = true;
		}
	});

	function handleClose() {
		if (pushed) {
			popRegion();
			pushed = false;
		}
		onclose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			handleClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- Backdrop -->
	<button
		class="fixed inset-0 z-50 bg-black/50 transition-opacity"
		onclick={handleClose}
		aria-label="Close panel"
		tabindex="-1"
	></button>

	<!-- Panel -->
	<div
		class="fixed top-0 right-0 z-50 flex h-full w-[60%] min-w-[400px] max-w-[900px] flex-col border-l border-border bg-background shadow-xl transition-transform duration-300"
		style="animation: slide-in 200ms ease-out"
		role="dialog"
		aria-modal="true"
		data-focus-region={focusRegion}
	>
		{@render children()}
	</div>
{/if}

<style>
	@keyframes slide-in {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}
</style>
