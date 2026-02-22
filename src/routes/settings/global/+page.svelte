<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import AgentProfiles from '$lib/components/settings/agent-profiles.svelte';
	import StatusGroupConfig from '$lib/components/settings/status-group-config.svelte';
	import {
		getGlobalConfig,
		updateGlobalConfig,
		getStatusGroupDefaults
	} from '$lib/services/config.js';
	import type {
		GlobalConfigResponse,
		StatusGroupConfigInput
	} from '$lib/types/index.js';
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import CheckIcon from '@lucide/svelte/icons/check';

	let globalConfig: GlobalConfigResponse | null = $state(null);
	let statusDefaults: Record<string, StatusGroupConfigInput> = $state({});
	let isLoading = $state(true);
	let error: string | null = $state(null);
	let savingBasePath = $state(false);
	let savedBasePath = $state(false);

	let basePathInput = $state('');

	onMount(() => {
		loadConfig();
	});

	async function loadConfig() {
		isLoading = true;
		error = null;
		try {
			const [config, defaults] = await Promise.all([
				getGlobalConfig(),
				getStatusGroupDefaults()
			]);
			globalConfig = config;
			statusDefaults = defaults.status;
			basePathInput = config.storage_base_path;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			isLoading = false;
		}
	}

	async function handleBasePathSave() {
		if (!globalConfig || savingBasePath) return;
		savingBasePath = true;
		error = null;
		try {
			const updated = await updateGlobalConfig({ storage_base_path: basePathInput.trim() });
			globalConfig = updated;
			savedBasePath = true;
			setTimeout(() => (savedBasePath = false), 1500);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			savingBasePath = false;
		}
	}

	async function handleDefaultAgentChange(agent: string) {
		if (!globalConfig) return;
		error = null;
		try {
			const updated = await updateGlobalConfig({ default_agent: agent });
			globalConfig = updated;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex h-14 items-center border-b border-border px-6">
		<a href={resolve('/settings')} class="mr-3">
			<Button variant="ghost" size="sm" class="h-7 w-7 p-0">
				<ArrowLeftIcon class="size-4" />
			</Button>
		</a>
		<h1 class="text-lg font-semibold">Global Settings</h1>
	</div>

	<div class="flex-1 overflow-y-auto p-6">
		<div class="mx-auto max-w-2xl">
			{#if isLoading}
				<p class="text-sm text-muted-foreground">Loading configuration...</p>
			{:else if error}
				<div
					class="mb-4 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
					role="alert"
				>
					{error}
					<button class="ml-2 underline" onclick={() => (error = null)}>Dismiss</button>
				</div>
			{:else if globalConfig}
				<section>
					<h2 class="mb-4 text-base font-semibold">Storage</h2>
					<p class="mb-4 text-sm text-muted-foreground">
						Configure the base path where Maestro stores project data, worktrees, and artifacts.
					</p>
					<div class="flex items-center gap-2">
						<Input
							bind:value={basePathInput}
							placeholder="~/.maestro"
							disabled={savingBasePath}
							class="flex-1"
						/>
						<Button
							variant="outline"
							size="sm"
							onclick={handleBasePathSave}
							disabled={savingBasePath || basePathInput === globalConfig.storage_base_path}
						>
							{#if savedBasePath}
								<CheckIcon class="size-4 text-green-600" />
							{:else}
								Save
							{/if}
						</Button>
					</div>
				</section>

				<Separator class="my-8" />

				<section>
					<h2 class="mb-4 text-base font-semibold">Default Agent</h2>
					<p class="mb-4 text-sm text-muted-foreground">
						Select the default agent to use when no project-level or status-group override is
						configured.
					</p>
					<Select.Root
						type="single"
						value={globalConfig.default_agent}
						onValueChange={handleDefaultAgentChange}
					>
						<Select.Trigger class="w-64">
							{globalConfig.default_agent}
						</Select.Trigger>
						<Select.Content>
							{#each globalConfig.agents as agent (agent.name)}
								<Select.Item value={agent.name}>{agent.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</section>

				<Separator class="my-8" />

				<section>
					<h2 class="mb-4 text-base font-semibold">Agent Profiles</h2>
					<p class="mb-4 text-sm text-muted-foreground">
						Configure agent profiles with custom binaries, flags, and environment variables.
					</p>
					<AgentProfiles agents={globalConfig.agents} onUpdate={loadConfig} />
				</section>

				<Separator class="my-8" />

				<section>
					<h2 class="mb-4 text-base font-semibold">Status Group Defaults</h2>
					<p class="mb-4 text-sm text-muted-foreground">
						Configure default agent, model, and instructions for each status group. These settings
						apply globally unless overridden at the project level.
					</p>
					<StatusGroupConfig
						agents={globalConfig.agents}
						{statusDefaults}
						defaultAgent={globalConfig.default_agent}
						onUpdate={loadConfig}
					/>
				</section>
			{/if}
		</div>
	</div>
</div>
