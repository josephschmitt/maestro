<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import AgentProfileForm from './agent-profile-form.svelte';
	import type { AgentProfileConfig, AgentProfileInput } from '$lib/types/index.js';
	import {
		createAgentProfile,
		updateAgentProfile,
		deleteAgentProfile
	} from '$lib/services/config.js';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import TerminalIcon from '@lucide/svelte/icons/terminal';

	let {
		agents,
		onUpdate
	}: {
		agents: AgentProfileConfig[];
		onUpdate: () => Promise<void>;
	} = $props();

	let dialogOpen = $state(false);
	let editingProfile: AgentProfileConfig | null = $state(null);
	let error: string | null = $state(null);

	const existingNames = $derived(agents.map((a) => a.name));

	function handleAdd() {
		editingProfile = null;
		dialogOpen = true;
	}

	function handleEdit(profile: AgentProfileConfig) {
		editingProfile = profile;
		dialogOpen = true;
	}

	async function handleDelete(name: string) {
		error = null;
		try {
			await deleteAgentProfile(name);
			await onUpdate();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleSave(profile: AgentProfileInput, originalName?: string) {
		if (originalName) {
			await updateAgentProfile(originalName, profile);
		} else {
			await createAgentProfile(profile);
		}
		await onUpdate();
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

{#if agents.length === 0}
	<p class="mb-3 text-sm text-muted-foreground">
		No agent profiles configured. Add a profile to get started.
	</p>
{:else}
	<div class="mb-3 space-y-2">
		{#each agents as agent (agent.name)}
			<div class="rounded-md border border-border px-3 py-2">
				<div class="flex items-start gap-2">
					<TerminalIcon class="mt-0.5 size-4 shrink-0 text-muted-foreground" />

					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="text-sm font-medium">{agent.name}</span>
						</div>
						<p class="text-xs text-muted-foreground">
							{#if agent.custom_command}
								<code class="rounded bg-muted px-1">{agent.custom_command}</code>
							{:else}
								<code class="rounded bg-muted px-1">{agent.binary}</code>
								{#if agent.flags.length > 0}
									{#each agent.flags as flag}
										<code class="ml-1 rounded bg-muted px-1">{flag}</code>
									{/each}
								{/if}
							{/if}
						</p>
						{#if agent.env_vars && Object.keys(agent.env_vars).length > 0}
							<p class="mt-1 text-xs text-muted-foreground">
								<span class="font-medium">Env:</span>
								{Object.keys(agent.env_vars).join(', ')}
							</p>
						{/if}
					</div>

					<div class="flex shrink-0 gap-1">
						<Button
							variant="ghost"
							size="sm"
							class="h-7 w-7 p-0"
							title="Edit profile"
							onclick={() => handleEdit(agent)}
						>
							<PencilIcon class="size-3.5" />
						</Button>
						<Button
							variant="ghost"
							size="sm"
							class="h-7 w-7 p-0 text-destructive hover:text-destructive"
							title="Delete profile"
							onclick={() => handleDelete(agent.name)}
						>
							<TrashIcon class="size-3.5" />
						</Button>
					</div>
				</div>
			</div>
		{/each}
	</div>
{/if}

<Button variant="outline" size="sm" onclick={handleAdd}>
	<PlusIcon class="size-3.5" />
	Add profile
</Button>

<AgentProfileForm
	bind:open={dialogOpen}
	profile={editingProfile}
	{existingNames}
	onSave={handleSave}
/>
