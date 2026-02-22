<script lang="ts">
	import type { Toast } from '$lib/stores/toasts.js';
	import { toasts } from '$lib/stores/toasts.js';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle-2';
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import AlertTriangleIcon from '@lucide/svelte/icons/alert-triangle';
	import InfoIcon from '@lucide/svelte/icons/info';
	import XIcon from '@lucide/svelte/icons/x';

	let { toast }: { toast: Toast } = $props();

	const iconMap = {
		success: CheckCircleIcon,
		error: AlertCircleIcon,
		warning: AlertTriangleIcon,
		info: InfoIcon,
	};

	const colorMap = {
		success: 'border-emerald-500 bg-emerald-50 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-200',
		error: 'border-red-500 bg-red-50 text-red-800 dark:bg-red-950 dark:text-red-200',
		warning: 'border-amber-500 bg-amber-50 text-amber-800 dark:bg-amber-950 dark:text-amber-200',
		info: 'border-blue-500 bg-blue-50 text-blue-800 dark:bg-blue-950 dark:text-blue-200',
	};

	const iconColorMap = {
		success: 'text-emerald-500',
		error: 'text-red-500',
		warning: 'text-amber-500',
		info: 'text-blue-500',
	};

	const Icon = iconMap[toast.type];
</script>

<div
	class="pointer-events-auto flex w-80 items-start gap-3 rounded-lg border-l-4 p-3 shadow-lg {colorMap[toast.type]}"
	role="alert"
	aria-live={toast.type === 'error' ? 'assertive' : 'polite'}
>
	<Icon class="size-5 shrink-0 {iconColorMap[toast.type]}" />
	<div class="min-w-0 flex-1">
		<p class="text-sm font-medium">{toast.title}</p>
		{#if toast.message}
			<p class="mt-0.5 text-xs opacity-80">{toast.message}</p>
		{/if}
	</div>
	<button
		class="shrink-0 rounded p-0.5 opacity-60 hover:opacity-100 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
		onclick={() => toasts.dismiss(toast.id)}
		aria-label="Dismiss"
	>
		<XIcon class="size-4" />
	</button>
</div>
