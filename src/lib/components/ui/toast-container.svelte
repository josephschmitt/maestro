<script lang="ts">
	import { toasts } from '$lib/stores/toasts.js';
	import Toast from './toast.svelte';
	import { fly } from 'svelte/transition';
</script>

<div
	class="pointer-events-none fixed right-4 top-4 z-[100] flex flex-col gap-2"
	aria-label="Notifications"
>
	{#each $toasts as toast (toast.id)}
		<div transition:fly={{ x: 100, duration: 200 }}>
			<Toast {toast} />
		</div>
	{/each}
</div>
