<script lang="ts">
	import type { Artifact } from '$lib/types/index.js';
	import {
		artifacts,
		loadArtifacts,
		addArtifact,
		removeArtifact
	} from '$lib/stores/artifacts.js';
	import ArtifactItem from '../artifact-item.svelte';
	import ArtifactEditor from '../artifact-editor.svelte';
	import ConfirmDialog from '$lib/components/ui/confirm-dialog.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';

	let {
		cardId
	}: {
		cardId: string;
	} = $props();

	let newArtifactName = $state('');
	let editingArtifactId = $state<string | null>(null);
	let confirmDeleteId = $state<string | null>(null);
	let confirmDialogOpen = $state(false);
	let deleteLoading = $state(false);

	$effect(() => {
		loadArtifacts(cardId);
		editingArtifactId = null;
	});

	let editingArtifact = $derived(
		editingArtifactId ? $artifacts.find((a) => a.id === editingArtifactId) ?? null : null
	);

	async function handleCreate() {
		const name = newArtifactName.trim();
		if (!name) return;
		const artifact = await addArtifact(cardId, name, '', 'user');
		newArtifactName = '';
		if (artifact) {
			editingArtifactId = artifact.id;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleCreate();
		}
	}

	function handleOpen(id: string) {
		editingArtifactId = id;
	}

	function handleDeleteClick(id: string) {
		confirmDeleteId = id;
		confirmDialogOpen = true;
	}

	async function handleConfirmDelete() {
		if (!confirmDeleteId) return;
		deleteLoading = true;
		await removeArtifact(confirmDeleteId, cardId);
		if (editingArtifactId === confirmDeleteId) {
			editingArtifactId = null;
		}
		confirmDeleteId = null;
		confirmDialogOpen = false;
		deleteLoading = false;
	}

	function handleCancelDelete() {
		confirmDeleteId = null;
		confirmDialogOpen = false;
	}
</script>

{#if editingArtifact}
	<ArtifactEditor
		artifact={editingArtifact}
		{cardId}
		onback={() => (editingArtifactId = null)}
	/>
{:else}
	<div class="flex flex-col gap-3">
		<!-- Create artifact form -->
		<div class="flex gap-2">
			<input
				type="text"
				class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				placeholder="New artifact name..."
				bind:value={newArtifactName}
				onkeydown={handleKeydown}
			/>
			<button
				class="inline-flex shrink-0 items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
				onclick={handleCreate}
				disabled={!newArtifactName.trim()}
				aria-label="Create artifact"
			>
				<PlusIcon size={14} class="mr-1" />
				Create
			</button>
		</div>

		<!-- Artifact list -->
		{#if $artifacts.length > 0}
			<div class="flex flex-col gap-0.5">
				{#each $artifacts as artifact (artifact.id)}
					<ArtifactItem
						{artifact}
						onclick={handleOpen}
						ondelete={handleDeleteClick}
					/>
				{/each}
			</div>
		{:else}
			<p class="py-4 text-center text-sm text-muted-foreground">
				No artifacts yet.
			</p>
		{/if}
	</div>

{/if}

<ConfirmDialog
	bind:open={confirmDialogOpen}
	title="Delete artifact?"
	message="This will permanently remove the artifact and its file from disk. This action cannot be undone."
	confirmLabel="Delete"
	loading={deleteLoading}
	onconfirm={handleConfirmDelete}
	oncancel={handleCancelDelete}
/>
