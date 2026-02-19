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
	import { createProject } from '$lib/stores/project.js';

	let {
		open = $bindable(false)
	}: {
		open: boolean;
	} = $props();

	let projectName = $state('');
	let isCreating = $state(false);

	async function handleCreate() {
		const name = projectName.trim();
		if (!name || isCreating) return;

		isCreating = true;
		try {
			await createProject(name);
			projectName = '';
			open = false;
		} finally {
			isCreating = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleCreate();
		}
	}
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Create project</DialogTitle>
			<DialogDescription>Give your project a name to get started.</DialogDescription>
		</DialogHeader>
		<div class="py-4">
			<Input
				bind:value={projectName}
				placeholder="Project name"
				onkeydown={handleKeydown}
				autofocus
			/>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
			<Button onclick={handleCreate} disabled={!projectName.trim() || isCreating}>
				{isCreating ? 'Creating...' : 'Create'}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
