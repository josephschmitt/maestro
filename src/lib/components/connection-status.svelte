<script lang="ts">
	import { connectionStatus } from '$lib/services/events.js';
	import { getTransportMode } from '$lib/services/db.js';
	import { onMount } from 'svelte';

	let isHttpMode = $state(false);

	onMount(() => {
		isHttpMode = getTransportMode() === 'http';
	});

	const statusConfig: Record<string, { color: string; pulse: boolean; label: string; description: string }> = {
		connected: {
			color: 'bg-green-500',
			pulse: false,
			label: 'Connected',
			description: 'Live connection to server'
		},
		reconnecting: {
			color: 'bg-yellow-500',
			pulse: true,
			label: 'Reconnecting',
			description: 'Attempting to reconnect...'
		},
		disconnected: {
			color: 'bg-red-500',
			pulse: false,
			label: 'Disconnected',
			description: 'Connection lost, data may be stale'
		}
	};

	let config = $derived(statusConfig[$connectionStatus]);
</script>

{#if isHttpMode}
	<div class="fixed bottom-4 right-4 z-50" role="status" aria-live="polite">
		<div
			class="flex items-center gap-2 rounded-full border border-border bg-background/95 px-3 py-1.5 shadow-lg backdrop-blur-sm"
		>
			<span
				class="size-2 rounded-full {config.color} {config.pulse ? 'animate-pulse' : ''}"
				aria-hidden="true"
			></span>
			<span class="text-xs font-medium">{config.label}</span>
		</div>
	</div>
{/if}
