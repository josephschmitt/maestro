<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { STATUS_GROUPS, type StatusGroup } from '$lib/types/status.js';
	import type { AgentProfileConfig, StatusGroupConfigInput } from '$lib/types/index.js';
	import { updateStatusGroupDefaults } from '$lib/services/config.js';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import CheckIcon from '@lucide/svelte/icons/check';

	let {
		agents,
		statusDefaults,
		defaultAgent,
		onUpdate
	}: {
		agents: AgentProfileConfig[];
		statusDefaults: Record<string, StatusGroupConfigInput>;
		defaultAgent: string;
		onUpdate: () => Promise<void>;
	} = $props();

	let error: string | null = $state(null);
	let savingGroup: string | null = $state(null);
	let savedGroup: string | null = $state(null);

	function getGroupConfig(group: StatusGroup): StatusGroupConfigInput {
		const key = group.toLowerCase();
		return statusDefaults[key] ?? { agent: null, model: null, instructions: null };
	}

	function getSelectedAgent(group: StatusGroup): string {
		const config = getGroupConfig(group);
		return config.agent ?? '';
	}

	function getModel(group: StatusGroup): string {
		const config = getGroupConfig(group);
		return config.model ?? '';
	}

	function getInstructions(group: StatusGroup): string {
		const config = getGroupConfig(group);
		return config.instructions ?? '';
	}

	async function handleUpdate(group: StatusGroup, field: keyof StatusGroupConfigInput, value: string) {
		error = null;
		savingGroup = group;

		const currentConfig = getGroupConfig(group);
		const updatedConfig: StatusGroupConfigInput = {
			agent: currentConfig.agent,
			model: currentConfig.model,
			instructions: currentConfig.instructions
		};

		if (field === 'agent') {
			updatedConfig.agent = value || null;
		} else if (field === 'model') {
			updatedConfig.model = value || null;
		} else if (field === 'instructions') {
			updatedConfig.instructions = value || null;
		}

		try {
			await updateStatusGroupDefaults(group, updatedConfig);
			await onUpdate();
			savedGroup = group;
			setTimeout(() => {
				if (savedGroup === group) {
					savedGroup = null;
				}
			}, 1500);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			savingGroup = null;
		}
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

<div class="space-y-4">
	{#each STATUS_GROUPS as group (group)}
		{@const config = getGroupConfig(group)}
		{@const isSaving = savingGroup === group}
		{@const isSaved = savedGroup === group}
		<div class="rounded-md border border-border p-4">
			<div class="mb-3 flex items-center gap-2">
				<SettingsIcon class="size-4 text-muted-foreground" />
				<h4 class="text-sm font-semibold">{group}</h4>
				{#if isSaved}
					<CheckIcon class="size-4 text-green-600" />
				{/if}
			</div>
			<p class="mb-3 text-xs text-muted-foreground">{getGroupDescription(group)}</p>

			<div class="space-y-3">
				<div>
					<label for="agent-{group}" class="mb-1 block text-xs font-medium text-muted-foreground">
						Agent
					</label>
					<Select.Root
						type="single"
						value={getSelectedAgent(group)}
						onValueChange={(v) => handleUpdate(group, 'agent', v ?? '')}
						disabled={isSaving}
					>
						<Select.Trigger id="agent-{group}" class="w-full">
							{#if config.agent}
								{config.agent}
							{:else}
								<span class="text-muted-foreground">Use default ({defaultAgent})</span>
							{/if}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="">Use default ({defaultAgent})</Select.Item>
							{#each agents as agent (agent.name)}
								<Select.Item value={agent.name}>{agent.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div>
					<label for="model-{group}" class="mb-1 block text-xs font-medium text-muted-foreground">
						Model
					</label>
					<Input
						id="model-{group}"
						value={getModel(group)}
						placeholder="e.g., sonnet, opus"
						disabled={isSaving}
						onblur={(e) => {
							const value = (e.target as HTMLInputElement).value;
							if (value !== getModel(group)) {
								handleUpdate(group, 'model', value);
							}
						}}
					/>
				</div>

				<div>
					<label
						for="instructions-{group}"
						class="mb-1 block text-xs font-medium text-muted-foreground"
					>
						Instructions
					</label>
					<textarea
						id="instructions-{group}"
						class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 flex min-h-[80px] w-full rounded-md border px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50"
						value={getInstructions(group)}
						placeholder="Custom instructions for this status group..."
						disabled={isSaving}
						onblur={(e) => {
							const value = (e.target as HTMLTextAreaElement).value;
							if (value !== getInstructions(group)) {
								handleUpdate(group, 'instructions', value);
							}
						}}
					></textarea>
				</div>
			</div>
		</div>
	{/each}
</div>
