<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { authStore, requiresAuth, hasAuthToken } from '$lib/stores/auth.js';
	import { getAuthToken } from '$lib/services/db.js';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import CheckIcon from '@lucide/svelte/icons/check';

	let currentToken = $state('');

	$effect(() => {
		if ($hasAuthToken) {
			const token = getAuthToken();
			if (token) {
				currentToken = token.substring(0, 8) + '...' + token.substring(token.length - 4);
			}
		} else {
			currentToken = '';
		}
	});

	function handleClear() {
		authStore.clearToken();
	}

	function handleChange() {
		authStore.openPrompt();
	}
</script>

{#if $requiresAuth}
	<div class="rounded-lg border border-border p-4">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-3">
				<div class="flex size-10 items-center justify-center rounded-lg bg-muted">
					<KeyRoundIcon class="size-5 text-muted-foreground" />
				</div>
				<div>
					<p class="text-sm font-medium">Authentication Token</p>
					{#if $hasAuthToken}
						<p class="flex items-center gap-1.5 text-xs text-muted-foreground">
							<CheckIcon class="size-3 text-green-500" />
							Connected
							<span class="font-mono">{currentToken}</span>
						</p>
					{:else}
						<p class="text-xs text-muted-foreground">No token configured</p>
					{/if}
				</div>
			</div>
			<div class="flex gap-2">
				{#if $hasAuthToken}
					<Button variant="outline" size="sm" onclick={handleChange}>Change</Button>
					<Button variant="outline" size="sm" onclick={handleClear}>Clear</Button>
				{:else}
					<Button size="sm" onclick={handleChange}>Configure</Button>
				{/if}
			</div>
		</div>
	</div>
{:else}
	<div class="rounded-lg border border-border bg-muted/30 p-4">
		<p class="text-sm text-muted-foreground">
			Authentication is only required when connecting to a remote Maestro server over HTTP.
		</p>
	</div>
{/if}
