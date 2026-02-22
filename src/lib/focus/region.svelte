<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { FocusRegionId } from './types.js';
	import { setActiveRegion } from './context.js';

	let {
		region,
		children
	}: {
		region: FocusRegionId;
		children: Snippet;
	} = $props();

	function handleFocusIn() {
		setActiveRegion(region);
	}
</script>

<div data-focus-region={region} onfocusin={handleFocusIn} style="display: contents">
	{@render children()}
</div>
