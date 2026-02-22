<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import ConfirmDialog from '$lib/components/ui/confirm-dialog.svelte';
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

	let dialogOpen = $state(false);
	let confirmDialogOpen = $state(false);
	let deleteTargetId = $state<string | null>(null);
	let deleteLoading = $state(false);

	onMount(() => {
		loadLinkedDirectories();
	});

	function handleRemoveClick(id: string) {
		deleteTargetId = id;
		confirmDialogOpen = true;
	}

	async function handleConfirmRemove() {
		if (!deleteTargetId) return;
		deleteLoading = true;
		await removeLinkedDirectory(deleteTargetId);
		deleteTargetId = null;
		confirmDialogOpen = false;
		deleteLoading = false;
	}

	function handleCancelRemove() {
		deleteTargetId = null;
		confirmDialogOpen = false;
	}
</script>

{#if !$currentProject}
	<p class="text-sm text-muted-foreground">Select a project to manage linked directories.</p>
{:else}
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
						onclick={() => handleRemoveClick(dir.id)}
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

<ConfirmDialog
	bind:open={confirmDialogOpen}
	title="Remove linked directory?"
	message="This will unlink the directory from this project. The actual files on disk will not be affected."
	confirmLabel="Remove"
	loading={deleteLoading}
	onconfirm={handleConfirmRemove}
	oncancel={handleCancelRemove}
/>
