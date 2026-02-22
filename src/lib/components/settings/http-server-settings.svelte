<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from '$lib/components/ui/select/index.js';
	import {
		getHttpServerConfig,
		updateHttpServerConfig,
		regenerateAuthToken
	} from '$lib/services/config.js';
	import { getTransportMode } from '$lib/services/db.js';
	import type { HttpServerConfigResponse } from '$lib/types/index.js';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import CheckIcon from '@lucide/svelte/icons/check';
	import AlertTriangleIcon from '@lucide/svelte/icons/alert-triangle';
	import ServerIcon from '@lucide/svelte/icons/server';
	import { onMount } from 'svelte';

	let config = $state<HttpServerConfigResponse | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let portInput = $state('');
	let pendingChanges = $state(false);
	let copiedToken = $state(false);
	let copiedUrl = $state(false);
	let regenerating = $state(false);
	let isTauriMode = $state(false);

	onMount(() => {
		isTauriMode = getTransportMode() === 'tauri';
		loadConfig();
	});

	async function loadConfig() {
		loading = true;
		error = null;
		try {
			config = await getHttpServerConfig();
			portInput = String(config.port);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load config';
		} finally {
			loading = false;
		}
	}

	async function handlePortChange() {
		if (!config) return;
		const newPort = parseInt(portInput, 10);
		if (isNaN(newPort) || newPort < 1 || newPort > 65535) {
			error = 'Port must be between 1 and 65535';
			return;
		}
		if (newPort === config.port) return;

		error = null;
		try {
			config = await updateHttpServerConfig({ port: newPort });
			pendingChanges = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update port';
		}
	}

	async function handleBindAddressChange(value: string) {
		if (!config || value === config.bind_address) return;

		error = null;
		try {
			config = await updateHttpServerConfig({ bind_address: value });
			pendingChanges = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update bind address';
		}
	}

	async function handleCopyToken() {
		if (!config?.auth_token) return;
		await navigator.clipboard.writeText(config.auth_token);
		copiedToken = true;
		setTimeout(() => (copiedToken = false), 2000);
	}

	async function handleCopyUrl() {
		if (!config?.server_url) return;
		await navigator.clipboard.writeText(config.server_url);
		copiedUrl = true;
		setTimeout(() => (copiedUrl = false), 2000);
	}

	async function handleRegenerateToken() {
		regenerating = true;
		error = null;
		try {
			const newToken = await regenerateAuthToken();
			if (config) {
				config = { ...config, auth_token: newToken };
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to regenerate token';
		} finally {
			regenerating = false;
		}
	}

	function getMaskedToken(token: string): string {
		if (token.length <= 12) return token;
		return token.substring(0, 8) + '...' + token.substring(token.length - 4);
	}
</script>

{#if !isTauriMode}
	<div class="rounded-lg border border-border bg-muted/30 p-4">
		<p class="text-sm text-muted-foreground">
			HTTP server settings are only available in the desktop app.
		</p>
	</div>
{:else if loading}
	<div class="rounded-lg border border-border p-4">
		<p class="text-sm text-muted-foreground">Loading...</p>
	</div>
{:else if error && !config}
	<div class="rounded-lg border border-destructive p-4">
		<p class="text-sm text-destructive">{error}</p>
	</div>
{:else if config}
	<div class="space-y-4">
		{#if config.bind_address === '0.0.0.0'}
			<div
				class="flex items-start gap-3 rounded-lg border border-yellow-500/50 bg-yellow-500/10 p-4"
			>
				<AlertTriangleIcon class="mt-0.5 size-5 shrink-0 text-yellow-500" />
				<div>
					<p class="text-sm font-medium text-yellow-500">Network Mode Active</p>
					<p class="text-sm text-muted-foreground">
						The HTTP server is accessible from other devices on your network. Make sure you trust
						your network.
					</p>
				</div>
			</div>
		{/if}

		{#if pendingChanges}
			<div class="flex items-start gap-3 rounded-lg border border-blue-500/50 bg-blue-500/10 p-4">
				<ServerIcon class="mt-0.5 size-5 shrink-0 text-blue-500" />
				<div>
					<p class="text-sm font-medium text-blue-500">Restart Required</p>
					<p class="text-sm text-muted-foreground">
						Configuration changes will take effect after restarting the app.
					</p>
				</div>
			</div>
		{/if}

		{#if error}
			<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
				<p class="text-sm text-destructive">{error}</p>
			</div>
		{/if}

		<div class="rounded-lg border border-border p-4">
			<div class="space-y-4">
				<div class="grid gap-4 sm:grid-cols-2">
					<div>
						<label for="port" class="mb-1.5 block text-sm font-medium">Port</label>
						<Input
							id="port"
							type="number"
							bind:value={portInput}
							onblur={handlePortChange}
							onkeydown={(e) => e.key === 'Enter' && handlePortChange()}
							min="1"
							max="65535"
							class="w-full"
						/>
					</div>

					<div>
						<label for="bind-address" class="mb-1.5 block text-sm font-medium">Bind Address</label>
						<Select
							type="single"
							value={config.bind_address}
							onValueChange={handleBindAddressChange}
						>
							<SelectTrigger class="w-full">
								<span data-slot="select-value">
									{config.bind_address === '127.0.0.1'
										? 'This machine only'
										: 'All network interfaces'}
								</span>
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="127.0.0.1" label="This machine only (127.0.0.1)" />
								<SelectItem value="0.0.0.0" label="All network interfaces (0.0.0.0)" />
							</SelectContent>
						</Select>
					</div>
				</div>

				<div>
					<label class="mb-1.5 block text-sm font-medium">Server URL</label>
					<div class="flex items-center gap-2">
						<div
							class="flex-1 rounded-md border border-border bg-muted/50 px-3 py-2 font-mono text-sm"
						>
							{config.server_url}
						</div>
						<Button variant="outline" size="sm" onclick={handleCopyUrl}>
							{#if copiedUrl}
								<CheckIcon class="size-4 text-green-500" />
							{:else}
								<CopyIcon class="size-4" />
							{/if}
						</Button>
					</div>
				</div>

				{#if config.requires_auth}
					<div>
						<label class="mb-1.5 block text-sm font-medium">Auth Token</label>
						<div class="flex items-center gap-2">
							<div
								class="flex-1 rounded-md border border-border bg-muted/50 px-3 py-2 font-mono text-sm"
							>
								{getMaskedToken(config.auth_token)}
							</div>
							<Button variant="outline" size="sm" onclick={handleCopyToken}>
								{#if copiedToken}
									<CheckIcon class="size-4 text-green-500" />
								{:else}
									<CopyIcon class="size-4" />
								{/if}
							</Button>
							<Button variant="outline" size="sm" onclick={handleRegenerateToken} disabled={regenerating}>
								<RefreshCwIcon class="size-4 {regenerating ? 'animate-spin' : ''}" />
							</Button>
						</div>
						<p class="mt-1.5 text-xs text-muted-foreground">
							Use this token to authenticate when connecting from another device.
						</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
