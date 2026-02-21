<script lang="ts">
	import type { StatusGroup } from '$lib/types/index.js';
	import { STATUS_GROUPS } from '$lib/types/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import {
		statusesByGroup,
		addStatus,
		updateStatus,
		removeStatus,
		reorderStatuses,
		loadStatuses
	} from '$lib/stores/statuses.js';
	import { currentProject } from '$lib/stores/project.js';
	import { onMount } from 'svelte';
	import StatusPromptPicker from '$lib/components/settings/status-prompt-picker.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import GripVerticalIcon from '@lucide/svelte/icons/grip-vertical';
	import ChevronUpIcon from '@lucide/svelte/icons/chevron-up';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import StarIcon from '@lucide/svelte/icons/star';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import CheckIcon from '@lucide/svelte/icons/check';
	import XIcon from '@lucide/svelte/icons/x';

	const groupLabels: Record<StatusGroup, string> = {
		Backlog: 'Backlog',
		Unstarted: 'Unstarted',
		Started: 'Started',
		Completed: 'Completed',
		Cancelled: 'Cancelled'
	};

	const groupDescriptions: Record<StatusGroup, string> = {
		Backlog: 'Items not yet prioritized',
		Unstarted: 'Ready to be worked on',
		Started: 'Currently in progress',
		Completed: 'Finished work',
		Cancelled: 'Work that was abandoned'
	};

	let newStatusName: Record<string, string> = $state({});
	let editingId: string | null = $state(null);
	let editingName: string = $state('');
	let error: string | null = $state(null);

	onMount(() => {
		loadStatuses();
	});

	async function handleAddStatus(group: StatusGroup) {
		const name = newStatusName[group]?.trim();
		if (!name) return;

		try {
			error = null;
			await addStatus(group, name);
			newStatusName[group] = '';
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleDelete(id: string) {
		try {
			error = null;
			await removeStatus(id);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleSetDefault(id: string) {
		try {
			error = null;
			await updateStatus(id, { isDefault: true });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	function startEditing(id: string, currentName: string) {
		editingId = id;
		editingName = currentName;
	}

	async function saveEdit() {
		if (!editingId || !editingName.trim()) return;

		try {
			error = null;
			await updateStatus(editingId, { name: editingName.trim() });
			editingId = null;
			editingName = '';
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	function cancelEdit() {
		editingId = null;
		editingName = '';
	}

	async function handleUpdatePrompts(id: string, prompts: string[]) {
		try {
			error = null;
			await updateStatus(id, { statusPrompts: prompts });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleMoveUp(group: StatusGroup, index: number) {
		const groupStatuses = $statusesByGroup.get(group);
		if (!groupStatuses || index <= 0) return;

		const ids = groupStatuses.map((s) => s.id);
		[ids[index - 1], ids[index]] = [ids[index], ids[index - 1]];

		try {
			error = null;
			await reorderStatuses(group, ids);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleMoveDown(group: StatusGroup, index: number) {
		const groupStatuses = $statusesByGroup.get(group);
		if (!groupStatuses || index >= groupStatuses.length - 1) return;

		const ids = groupStatuses.map((s) => s.id);
		[ids[index], ids[index + 1]] = [ids[index + 1], ids[index]];

		try {
			error = null;
			await reorderStatuses(group, ids);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}
</script>

{#if !$currentProject}
	<p class="text-sm text-muted-foreground">Select a project to manage statuses.</p>
{:else}
	{#if error}
		<div
			class="mb-4 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
			role="alert"
		>
			{error}
			<button class="ml-2 underline" onclick={() => (error = null)}>Dismiss</button>
		</div>
	{/if}

	<div class="space-y-6">
		{#each STATUS_GROUPS as group (group)}
			{@const groupStatuses = $statusesByGroup.get(group) ?? []}
			<div>
				<div class="mb-2 flex items-center gap-2">
					<h3 class="text-sm font-semibold">{groupLabels[group]}</h3>
					<span class="text-xs text-muted-foreground">{groupDescriptions[group]}</span>
				</div>

				<div class="space-y-1">
					{#each groupStatuses as status, i (status.id)}
						<div
							class="rounded-md border border-border px-3 py-2"
						>
							<div class="flex items-center gap-2">
								<GripVerticalIcon class="size-4 shrink-0 text-muted-foreground" />

								{#if editingId === status.id}
									<Input
										value={editingName}
										oninput={(e: Event) => (editingName = (e.target as HTMLInputElement).value)}
										onkeydown={(e: KeyboardEvent) => {
											if (e.key === 'Enter') saveEdit();
											if (e.key === 'Escape') cancelEdit();
										}}
										class="h-7 flex-1 text-sm"
									/>
									<Button variant="ghost" size="sm" class="h-7 w-7 p-0" onclick={saveEdit}>
										<CheckIcon class="size-3.5" />
									</Button>
									<Button variant="ghost" size="sm" class="h-7 w-7 p-0" onclick={cancelEdit}>
										<XIcon class="size-3.5" />
									</Button>
								{:else}
									<StatusBadge name={status.name} {group} />

									{#if status.is_default}
										<StarIcon class="size-3.5 fill-amber-400 text-amber-400" />
									{/if}

									<div class="flex-1"></div>

									<Button
										variant="ghost"
										size="sm"
										class="h-7 w-7 p-0"
										title="Edit name"
										onclick={() => startEditing(status.id, status.name)}
									>
										<PencilIcon class="size-3.5" />
									</Button>

									{#if !status.is_default}
										<Button
											variant="ghost"
											size="sm"
											class="h-7 w-7 p-0"
											title="Set as default"
											onclick={() => handleSetDefault(status.id)}
										>
											<StarIcon class="size-3.5" />
										</Button>
									{/if}

									<Button
										variant="ghost"
										size="sm"
										class="h-7 w-7 p-0"
										disabled={i === 0}
										title="Move up"
										onclick={() => handleMoveUp(group, i)}
									>
										<ChevronUpIcon class="size-3.5" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										class="h-7 w-7 p-0"
										disabled={i === groupStatuses.length - 1}
										title="Move down"
										onclick={() => handleMoveDown(group, i)}
									>
										<ChevronDownIcon class="size-3.5" />
									</Button>

									<Button
										variant="ghost"
										size="sm"
										class="h-7 w-7 p-0 text-destructive hover:text-destructive"
										disabled={groupStatuses.length <= 1}
										title="Delete status"
										onclick={() => handleDelete(status.id)}
									>
										<TrashIcon class="size-3.5" />
									</Button>
								{/if}
							</div>

							{#if editingId !== status.id}
								<div class="ml-6 mt-1">
									<StatusPromptPicker
										statusPrompts={status.status_prompts}
										onchange={(prompts) => handleUpdatePrompts(status.id, prompts)}
									/>
								</div>
							{/if}
						</div>
					{/each}
				</div>

				<div class="mt-2 flex gap-2">
					<Input
						placeholder="New status name..."
						value={newStatusName[group] ?? ''}
						oninput={(e: Event) => (newStatusName[group] = (e.target as HTMLInputElement).value)}
						onkeydown={(e: KeyboardEvent) => {
							if (e.key === 'Enter') handleAddStatus(group);
						}}
						class="h-8 text-sm"
					/>
					<Button
						variant="outline"
						size="sm"
						class="h-8 shrink-0"
						onclick={() => handleAddStatus(group)}
					>
						<PlusIcon class="size-3.5" />
						Add
					</Button>
				</div>

				{#if group !== 'Cancelled'}
					<Separator class="mt-4" />
				{/if}
			</div>
		{/each}
	</div>
{/if}
