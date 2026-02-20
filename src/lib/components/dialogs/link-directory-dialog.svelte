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
	import { addLinkedDirectory } from '$lib/stores/directories.js';
	import FolderOpenIcon from '@lucide/svelte/icons/folder-open';

	let {
		open = $bindable(false)
	}: {
		open: boolean;
	} = $props();

	let path = $state('');
	let label = $state('');
	let isAdding = $state(false);
	let error = $state('');

	function defaultLabel(dirPath: string): string {
		const trimmed = dirPath.replace(/\/+$/, '');
		const parts = trimmed.split('/');
		return parts[parts.length - 1] || '';
	}

	async function handleBrowse() {
		try {
			const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
			const selected = await openDialog({ directory: true, multiple: false });
			if (selected) {
				path = selected as string;
				if (!label) {
					label = defaultLabel(path);
				}
			}
		} catch {
			// Not in Tauri environment, user can type path manually
		}
	}

	function handlePathInput(e: Event) {
		path = (e.target as HTMLInputElement).value;
		if (!label) {
			label = defaultLabel(path);
		}
	}

	async function handleAdd() {
		const trimmedPath = path.trim();
		const trimmedLabel = label.trim();
		if (!trimmedPath || !trimmedLabel || isAdding) return;

		isAdding = true;
		error = '';
		try {
			await addLinkedDirectory(trimmedPath, trimmedLabel);
			path = '';
			label = '';
			open = false;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			isAdding = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleAdd();
		}
	}
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Link directory</DialogTitle>
			<DialogDescription>
				Link an external directory (git repo, docs folder) to this project.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			{#if error}
				<div
					class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
					role="alert"
				>
					{error}
				</div>
			{/if}
			<div>
				<label for="dir-path" class="mb-1.5 block text-sm font-medium">Directory path</label>
				<div class="flex gap-2">
					<Input
						id="dir-path"
						value={path}
						oninput={handlePathInput}
						onkeydown={handleKeydown}
						placeholder="/path/to/directory"
						class="flex-1"
					/>
					<Button variant="outline" onclick={handleBrowse} title="Browse for directory">
						<FolderOpenIcon class="size-4" />
						Browse
					</Button>
				</div>
			</div>
			<div>
				<label for="dir-label" class="mb-1.5 block text-sm font-medium">Label</label>
				<Input
					id="dir-label"
					bind:value={label}
					onkeydown={handleKeydown}
					placeholder="Directory label"
				/>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
			<Button onclick={handleAdd} disabled={!path.trim() || !label.trim() || isAdding}>
				{isAdding ? 'Adding...' : 'Add'}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
