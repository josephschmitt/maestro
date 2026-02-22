<script lang="ts">
	import type { Snippet } from 'svelte';
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import { Button } from '$lib/components/ui/button/index.js';

	let { children }: { children: Snippet } = $props();

	let error = $state<Error | null>(null);
	let showDetails = $state(false);

	function handleError(e: ErrorEvent) {
		error = e.error instanceof Error ? e.error : new Error(String(e.error ?? e.message));
	}

	function handleUnhandledRejection(e: PromiseRejectionEvent) {
		error = e.reason instanceof Error ? e.reason : new Error(String(e.reason));
	}

	function handleReload() {
		window.location.reload();
	}

	function handleDismiss() {
		error = null;
	}

	$effect(() => {
		window.addEventListener('error', handleError);
		window.addEventListener('unhandledrejection', handleUnhandledRejection);

		return () => {
			window.removeEventListener('error', handleError);
			window.removeEventListener('unhandledrejection', handleUnhandledRejection);
		};
	});
</script>

{#if error}
	<div class="flex h-full w-full items-center justify-center bg-background p-8">
		<div class="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg">
			<div class="flex items-start gap-3">
				<div class="rounded-full bg-red-100 p-2 dark:bg-red-900/30">
					<AlertCircleIcon class="size-6 text-red-600 dark:text-red-400" />
				</div>
				<div class="flex-1">
					<h2 class="text-lg font-semibold text-foreground">Something went wrong</h2>
					<p class="mt-1 text-sm text-muted-foreground">
						An unexpected error occurred. You can try reloading the page or dismissing this error.
					</p>
				</div>
			</div>

			<div class="mt-4">
				<button
					class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
					onclick={() => (showDetails = !showDetails)}
				>
					{#if showDetails}
						<ChevronDownIcon class="size-4" />
					{:else}
						<ChevronRightIcon class="size-4" />
					{/if}
					Show error details
				</button>

				{#if showDetails}
					<div class="mt-2 rounded-md bg-muted p-3">
						<p class="text-sm font-medium text-foreground">{error.name}</p>
						<p class="mt-1 text-sm text-muted-foreground">{error.message}</p>
						{#if error.stack}
							<pre class="mt-2 max-h-40 overflow-auto text-xs text-muted-foreground">{error.stack}</pre>
						{/if}
					</div>
				{/if}
			</div>

			<div class="mt-4 flex gap-2">
				<Button variant="default" onclick={handleReload}>Reload page</Button>
				<Button variant="outline" onclick={handleDismiss}>Dismiss</Button>
			</div>
		</div>
	</div>
{:else}
	{@render children()}
{/if}
