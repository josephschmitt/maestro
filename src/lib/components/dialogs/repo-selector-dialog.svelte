<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import type { LinkedDirectory } from '$lib/types/index.js';
	import FolderGitIcon from '@lucide/svelte/icons/folder-git-2';

	let {
		open = $bindable(false),
		repos = [],
		onselect,
		onskip
	}: {
		open: boolean;
		repos: LinkedDirectory[];
		onselect: (repo: LinkedDirectory) => void;
		onskip: () => void;
	} = $props();

	function handleSelect(repo: LinkedDirectory) {
		open = false;
		onselect(repo);
	}

	function handleSkip() {
		open = false;
		onskip();
	}
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Select repository</DialogTitle>
			<DialogDescription>
				Choose which repository to create an implementation worktree in.
			</DialogDescription>
		</DialogHeader>
		<div class="flex flex-col gap-1 py-4">
			{#each repos as repo (repo.id)}
				<button
					class="flex items-center gap-3 rounded-md px-3 py-2.5 text-left hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					onclick={() => handleSelect(repo)}
				>
					<FolderGitIcon class="size-4 shrink-0 text-muted-foreground" />
					<div class="min-w-0 flex-1">
						<div class="truncate text-sm font-medium">{repo.label}</div>
						<div class="truncate text-xs text-muted-foreground">{repo.path}</div>
					</div>
				</button>
			{/each}
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={handleSkip}>
				Skip (use artifacts directory)
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
