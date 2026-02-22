<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { STATUS_GROUPS, type StatusGroup } from '$lib/types/status.js';
	import type {
		AgentProfileConfig,
		StatusGroupConfigInput,
		ResolvedAgentConfigResponse
	} from '$lib/types/index.js';
	import { resolveConfig } from '$lib/services/config.js';
	import { updateProject } from '$lib/services/projects.js';
	import { currentProject, reloadCurrentProject } from '$lib/stores/project.js';
	import { onMount } from 'svelte';
	import ToggleLeftIcon from '@lucide/svelte/icons/toggle-left';
	import ToggleRightIcon from '@lucide/svelte/icons/toggle-right';
	import CheckIcon from '@lucide/svelte/icons/check';

	let {
		agents,
		globalDefaults,
		globalDefaultAgent
	}: {
		agents: AgentProfileConfig[];
		globalDefaults: Record<string, StatusGroupConfigInput>;
		globalDefaultAgent: string;
	} = $props();

	interface ProjectAgentConfig {
		agent?: string;
		status?: Record<string, { agent?: string; model?: string; instructions?: string }>;
	}

	let error: string | null = $state(null);
	let savingGroup: string | null = $state(null);
	let savedGroup: string | null = $state(null);
	let savingDefaultAgent = $state(false);
	let savedDefaultAgent = $state(false);
	let resolvedConfigs: Record<string, ResolvedAgentConfigResponse> = $state({});

	let projectConfig: ProjectAgentConfig = $state({});

	$effect(() => {
		if ($currentProject) {
			projectConfig = ($currentProject.agent_config ?? {}) as ProjectAgentConfig;
			loadResolvedConfigs();
		}
	});

	async function loadResolvedConfigs() {
		if (!$currentProject) return;
		const results: Record<string, ResolvedAgentConfigResponse> = {};
		for (const group of STATUS_GROUPS) {
			try {
				results[group] = await resolveConfig($currentProject.agent_config ?? {}, group);
			} catch {
				results[group] = { agent: globalDefaultAgent, model: null, instructions: null };
			}
		}
		resolvedConfigs = results;
	}

	function getProjectDefaultAgent(): string | undefined {
		return projectConfig.agent;
	}

	function hasProjectDefaultAgentOverride(): boolean {
		return projectConfig.agent !== undefined;
	}

	function getProjectGroupConfig(group: StatusGroup): StatusGroupConfigInput | undefined {
		const key = group.toLowerCase();
		const status = projectConfig.status?.[key];
		if (!status) return undefined;
		return {
			agent: status.agent ?? null,
			model: status.model ?? null,
			instructions: status.instructions ?? null
		};
	}

	function hasGroupOverride(group: StatusGroup): boolean {
		const key = group.toLowerCase();
		const status = projectConfig.status?.[key];
		return status !== undefined && Boolean(status.agent || status.model || status.instructions);
	}

	function getGlobalGroupConfig(group: StatusGroup): StatusGroupConfigInput {
		const key = group.toLowerCase();
		return globalDefaults[key] ?? { agent: null, model: null, instructions: null };
	}

	function getResolvedAgent(group: StatusGroup): string {
		return resolvedConfigs[group]?.agent ?? globalDefaultAgent;
	}

	function getResolvedModel(group: StatusGroup): string | null {
		return resolvedConfigs[group]?.model ?? null;
	}

	function getResolvedInstructions(group: StatusGroup): string | null {
		return resolvedConfigs[group]?.instructions ?? null;
	}

	async function saveConfig(newConfig: ProjectAgentConfig) {
		if (!$currentProject) return;
		error = null;
		try {
			await updateProject($currentProject.id, { agent_config: newConfig as Record<string, unknown> });
			await reloadCurrentProject();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function toggleDefaultAgentOverride() {
		if (hasProjectDefaultAgentOverride()) {
			const { agent: _, ...rest } = projectConfig;
			projectConfig = rest;
		} else {
			projectConfig = { ...projectConfig, agent: globalDefaultAgent };
		}
		savingDefaultAgent = true;
		await saveConfig(projectConfig);
		savingDefaultAgent = false;
		savedDefaultAgent = true;
		setTimeout(() => (savedDefaultAgent = false), 1500);
	}

	async function handleDefaultAgentChange(value: string) {
		projectConfig = { ...projectConfig, agent: value };
		savingDefaultAgent = true;
		await saveConfig(projectConfig);
		savingDefaultAgent = false;
		savedDefaultAgent = true;
		setTimeout(() => (savedDefaultAgent = false), 1500);
	}

	async function toggleGroupOverride(group: StatusGroup) {
		const key = group.toLowerCase();
		if (hasGroupOverride(group)) {
			const newStatus = { ...projectConfig.status };
			delete newStatus[key];
			projectConfig = { ...projectConfig, status: newStatus };
		} else {
			const globalConfig = getGlobalGroupConfig(group);
			const newStatus = {
				...(projectConfig.status ?? {}),
				[key]: {
					agent: globalConfig.agent ?? undefined,
					model: globalConfig.model ?? undefined,
					instructions: globalConfig.instructions ?? undefined
				}
			};
			projectConfig = { ...projectConfig, status: newStatus };
		}
		savingGroup = group;
		await saveConfig(projectConfig);
		savingGroup = null;
		savedGroup = group;
		setTimeout(() => {
			if (savedGroup === group) savedGroup = null;
		}, 1500);
	}

	async function handleGroupFieldChange(
		group: StatusGroup,
		field: 'agent' | 'model' | 'instructions',
		value: string
	) {
		const key = group.toLowerCase();
		const currentStatus = projectConfig.status?.[key] ?? {};
		const newStatus = {
			...(projectConfig.status ?? {}),
			[key]: {
				...currentStatus,
				[field]: value || undefined
			}
		};
		projectConfig = { ...projectConfig, status: newStatus };
		savingGroup = group;
		await saveConfig(projectConfig);
		savingGroup = null;
		savedGroup = group;
		setTimeout(() => {
			if (savedGroup === group) savedGroup = null;
		}, 1500);
	}

	function getGroupDescription(group: StatusGroup): string {
		switch (group) {
			case 'Backlog':
				return 'Planning and ideation phase';
			case 'Unstarted':
				return 'Ready to begin work';
			case 'Started':
				return 'Active development';
			case 'Completed':
				return 'Work finished';
			case 'Cancelled':
				return 'Work abandoned';
			default:
				return '';
		}
	}
</script>

{#if error}
	<div
		class="mb-4 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
		role="alert"
	>
		{error}
		<button class="ml-2 underline" onclick={() => (error = null)}>Dismiss</button>
	</div>
{/if}

{#if !$currentProject}
	<p class="text-sm text-muted-foreground">Select a project to manage agent configuration.</p>
{:else}
	<div class="space-y-6">
		<div class="rounded-md border border-border p-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<h4 class="text-sm font-semibold">Default Agent</h4>
					{#if savedDefaultAgent}
						<CheckIcon class="size-4 text-green-600" />
					{/if}
				</div>
				<button
					class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
					onclick={toggleDefaultAgentOverride}
					disabled={savingDefaultAgent}
				>
					{#if hasProjectDefaultAgentOverride()}
						<ToggleRightIcon class="size-4 text-primary" />
						<span>Override</span>
					{:else}
						<ToggleLeftIcon class="size-4" />
						<span>Use global</span>
					{/if}
				</button>
			</div>
			<div class="mt-3">
				{#if hasProjectDefaultAgentOverride()}
					<Select.Root
						type="single"
						value={getProjectDefaultAgent()}
						onValueChange={handleDefaultAgentChange}
						disabled={savingDefaultAgent}
					>
						<Select.Trigger class="w-full">
							{getProjectDefaultAgent()}
						</Select.Trigger>
						<Select.Content>
							{#each agents as agent (agent.name)}
								<Select.Item value={agent.name}>{agent.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				{:else}
					<div class="rounded-md bg-muted/50 px-3 py-2 text-sm text-muted-foreground">
						Using global default: <span class="font-medium">{globalDefaultAgent}</span>
					</div>
				{/if}
			</div>
		</div>

		<h4 class="text-sm font-semibold">Status Group Overrides</h4>
		<p class="text-sm text-muted-foreground">
			Override agent, model, and instructions for specific status groups.
		</p>

		{#each STATUS_GROUPS as group (group)}
			{@const isOverriding = hasGroupOverride(group)}
			{@const isSaving = savingGroup === group}
			{@const isSaved = savedGroup === group}
			{@const projectGroupConfig = getProjectGroupConfig(group)}
			{@const globalGroupConfig = getGlobalGroupConfig(group)}

			<div class="rounded-md border border-border p-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h4 class="text-sm font-semibold">{group}</h4>
						{#if isSaved}
							<CheckIcon class="size-4 text-green-600" />
						{/if}
					</div>
					<button
						class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
						onclick={() => toggleGroupOverride(group)}
						disabled={isSaving}
					>
						{#if isOverriding}
							<ToggleRightIcon class="size-4 text-primary" />
							<span>Override</span>
						{:else}
							<ToggleLeftIcon class="size-4" />
							<span>Use global</span>
						{/if}
					</button>
				</div>
				<p class="mb-3 text-xs text-muted-foreground">{getGroupDescription(group)}</p>

				{#if isOverriding}
					<div class="space-y-3">
						<div>
							<label
								for="project-agent-{group}"
								class="mb-1 block text-xs font-medium text-muted-foreground"
							>
								Agent
							</label>
							<Select.Root
								type="single"
								value={projectGroupConfig?.agent ?? ''}
								onValueChange={(v) => handleGroupFieldChange(group, 'agent', v ?? '')}
								disabled={isSaving}
							>
								<Select.Trigger id="project-agent-{group}" class="w-full">
									{#if projectGroupConfig?.agent}
										{projectGroupConfig.agent}
									{:else}
										<span class="text-muted-foreground">Use default</span>
									{/if}
								</Select.Trigger>
								<Select.Content>
									<Select.Item value="">Use default</Select.Item>
									{#each agents as agent (agent.name)}
										<Select.Item value={agent.name}>{agent.name}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>

						<div>
							<label
								for="project-model-{group}"
								class="mb-1 block text-xs font-medium text-muted-foreground"
							>
								Model
							</label>
							<Input
								id="project-model-{group}"
								value={projectGroupConfig?.model ?? ''}
								placeholder="e.g., sonnet, opus"
								disabled={isSaving}
								onblur={(e) => {
									const value = (e.target as HTMLInputElement).value;
									if (value !== (projectGroupConfig?.model ?? '')) {
										handleGroupFieldChange(group, 'model', value);
									}
								}}
							/>
						</div>

						<div>
							<label
								for="project-instructions-{group}"
								class="mb-1 block text-xs font-medium text-muted-foreground"
							>
								Instructions
							</label>
							<textarea
								id="project-instructions-{group}"
								class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 flex min-h-[80px] w-full rounded-md border px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50"
								value={projectGroupConfig?.instructions ?? ''}
								placeholder="Custom instructions..."
								disabled={isSaving}
								onblur={(e) => {
									const value = (e.target as HTMLTextAreaElement).value;
									if (value !== (projectGroupConfig?.instructions ?? '')) {
										handleGroupFieldChange(group, 'instructions', value);
									}
								}}
							></textarea>
						</div>
					</div>
				{:else}
					<div class="rounded-md bg-muted/50 px-3 py-2 text-sm text-muted-foreground">
						<div class="space-y-1">
							<div>
								<span class="font-medium">Agent:</span>
								{globalGroupConfig.agent ?? globalDefaultAgent}
							</div>
							{#if globalGroupConfig.model}
								<div><span class="font-medium">Model:</span> {globalGroupConfig.model}</div>
							{/if}
							{#if globalGroupConfig.instructions}
								<div class="text-xs">
									<span class="font-medium">Instructions:</span> {globalGroupConfig.instructions.slice(0, 100)}{globalGroupConfig.instructions.length > 100 ? '...' : ''}
								</div>
							{/if}
						</div>
					</div>
				{/if}

				<div class="mt-3 border-t border-border pt-2">
					<p class="text-[10px] uppercase tracking-wider text-muted-foreground">Resolved</p>
					<p class="text-xs">
						<span class="font-medium">{getResolvedAgent(group)}</span>
						{#if getResolvedModel(group)}
							<span class="text-muted-foreground">/ {getResolvedModel(group)}</span>
						{/if}
					</p>
				</div>
			</div>
		{/each}
	</div>
{/if}
