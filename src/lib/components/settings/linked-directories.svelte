<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import {
		linkedDirectories,
		loadLinkedDirectories,
		removeLinkedDirectory
	} from '$lib/stores/directories.js';
	import { currentProject } from '$lib/stores/project.js';
	import { onMount } from 'svelte';
	import LinkDirectoryDialog from '$lib/components/dialogs/link-directory-dialog.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import GitBranchIcon from '@lucide/svelte/icons/git-branch';
	import FolderIcon from '@lucide/svelte/icons/folder';

	let error: string | null = $state(null);
	let dialogOpen = $state(false);

	onMount(() => {
		loadLinkedDirectories();
	});

	async function handleRemove(id: string) {
		try {
			error = null;
			await removeLinkedDirectory(id);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}
</script>

{#if !$currentProject}
	<p class="text-sm text-muted-foreground">Select a project to manage linked directories.</p>
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

	{#if $linkedDirectories.length === 0}
		<p class="mb-3 text-sm text-muted-foreground">
			No directories linked yet. Link a git repo or folder to get started.
		</p>
	{:else}
		<div class="mb-3 space-y-1">
			{#each $linkedDirectories as dir (dir.id)}
				<div class="flex items-center gap-2 rounded-md border border-border px-3 py-2">
					{#if dir.is_repo}
						<GitBranchIcon class="size-4 shrink-0 text-muted-foreground" />
					{:else}
						<FolderIcon class="size-4 shrink-0 text-muted-foreground" />
					{/if}

					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="text-sm font-medium">{dir.label}</span>
							{#if dir.is_repo}
								<span
									class="rounded bg-emerald-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-emerald-700 dark:bg-emerald-900 dark:text-emerald-300"
								>
									git
								</span>
							{/if}
						</div>
						<p class="truncate text-xs text-muted-foreground" title={dir.path}>
							{dir.path}
						</p>
					</div>

					<Button
						variant="ghost"
						size="sm"
						class="h-7 w-7 shrink-0 p-0 text-destructive hover:text-destructive"
						title="Remove linked directory"
						onclick={() => handleRemove(dir.id)}
					>
						<TrashIcon class="size-3.5" />
					</Button>
				</div>
			{/each}
		</div>
	{/if}

	<Button variant="outline" size="sm" onclick={() => (dialogOpen = true)}>
		<PlusIcon class="size-3.5" />
		Link directory
	</Button>

	<LinkDirectoryDialog bind:open={dialogOpen} />
{/if}
