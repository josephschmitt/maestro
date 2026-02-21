<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import GitBranchIcon from '@lucide/svelte/icons/git-branch';

	let {
		open = $bindable(false),
		defaultBranchName = '',
		onconfirm,
		oncancel
	}: {
		open: boolean;
		defaultBranchName: string;
		onconfirm: (branchName: string) => void;
		oncancel: () => void;
	} = $props();

	let branchName = $state('');

	$effect(() => {
		if (open) {
			branchName = defaultBranchName;
		}
	});

	function handleConfirm() {
		const trimmed = branchName.trim();
		if (!trimmed) return;
		open = false;
		onconfirm(trimmed);
	}

	function handleCancel() {
		open = false;
		oncancel();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleConfirm();
		}
	}
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Confirm branch name</DialogTitle>
			<DialogDescription>
				A git worktree will be created with this branch for implementation.
			</DialogDescription>
		</DialogHeader>
		<div class="py-4">
			<label for="branch-name" class="mb-1.5 flex items-center gap-2 text-sm font-medium">
				<GitBranchIcon class="size-4" />
				Branch name
			</label>
			<Input
				id="branch-name"
				bind:value={branchName}
				onkeydown={handleKeydown}
				placeholder="maestro/..."
				class="font-mono text-sm"
			/>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={handleCancel}>Cancel</Button>
			<Button onclick={handleConfirm} disabled={!branchName.trim()}>
				Create worktree
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
