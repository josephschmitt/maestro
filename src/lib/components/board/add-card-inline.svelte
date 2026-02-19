<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';

	let {
		onAdd
	}: {
		onAdd: (title: string) => void;
	} = $props();

	let editing = $state(false);
	let title = $state('');
	let inputEl: HTMLInputElement | null = $state(null);

	function startEditing() {
		editing = true;
		title = '';
		requestAnimationFrame(() => {
			inputEl?.focus();
		});
	}

	function submit() {
		const trimmed = title.trim();
		if (trimmed) {
			onAdd(trimmed);
		}
		cancel();
	}

	function cancel() {
		editing = false;
		title = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			submit();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancel();
		}
	}
</script>

{#if editing}
	<div class="px-1 pb-1">
		<input
			bind:this={inputEl}
			bind:value={title}
			onkeydown={handleKeydown}
			onblur={submit}
			class="w-full rounded-md border border-input bg-background px-2.5 py-1.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
			placeholder="Card title..."
		/>
	</div>
{:else}
	<button
		onclick={startEditing}
		class="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
		tabindex="0"
	>
		<PlusIcon class="size-3.5" />
		Add card
	</button>
{/if}
