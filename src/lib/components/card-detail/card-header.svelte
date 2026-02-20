<script lang="ts">
	import type { CardWithStatus, Status } from '$lib/types/index.js';
	import StatusSelector from './status-selector.svelte';
	import LabelEditor from './label-editor.svelte';

	let {
		card,
		statuses,
		ontitlechange,
		onstatuschange,
		onlabelschange
	}: {
		card: CardWithStatus;
		statuses: Status[];
		ontitlechange: (title: string) => void;
		onstatuschange: (statusId: string) => void;
		onlabelschange: (labels: string[]) => void;
	} = $props();

	let editing = $state(false);
	let editValue = $state('');

	function startEditing() {
		editValue = card.title;
		editing = true;
	}

	function save() {
		const trimmed = editValue.trim();
		if (trimmed && trimmed !== card.title) {
			ontitlechange(trimmed);
		}
		editing = false;
	}

	function cancel() {
		editing = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			save();
		} else if (e.key === 'Escape') {
			cancel();
		}
	}
</script>

<div class="flex flex-col gap-4">
	<!-- Title -->
	<div>
		{#if editing}
			<input
				type="text"
				bind:value={editValue}
				onblur={save}
				onkeydown={handleKeydown}
				class="w-full rounded-md border border-input bg-transparent px-2 py-1 text-xl font-semibold focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
				autofocus
			/>
		{:else}
			<button
				class="w-full cursor-text text-left text-xl font-semibold text-foreground hover:text-foreground/80"
				onclick={startEditing}
				aria-label="Edit title"
			>
				{card.title}
			</button>
		{/if}
	</div>

	<!-- Status + Labels row -->
	<div class="flex flex-col gap-3">
		<div class="flex items-center gap-3">
			<span class="w-16 text-xs text-muted-foreground">Status</span>
			<StatusSelector
				{statuses}
				currentStatusId={card.status_id}
				onchange={onstatuschange}
			/>
		</div>

		<div class="flex items-start gap-3">
			<span class="mt-1 w-16 text-xs text-muted-foreground">Labels</span>
			<div class="flex-1">
				<LabelEditor labels={card.labels} onchange={onlabelschange} />
			</div>
		</div>
	</div>
</div>
