<script lang="ts">
	import type { Artifact } from '$lib/types/index.js';
	import { getArtifactContent, saveArtifactContent } from '$lib/stores/artifacts.js';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';

	let {
		artifact,
		cardId,
		onback
	}: {
		artifact: Artifact;
		cardId: string;
		onback: () => void;
	} = $props();

	let content = $state('');
	let loading = $state(true);
	let saving = $state(false);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		loadContent(artifact.id);
	});

	async function loadContent(id: string) {
		loading = true;
		try {
			content = await getArtifactContent(id);
		} catch {
			content = '';
		}
		loading = false;
	}

	function handleInput() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			save();
		}, 2000);
	}

	async function handleBlur() {
		if (debounceTimer) {
			clearTimeout(debounceTimer);
			debounceTimer = null;
		}
		await save();
	}

	async function save() {
		if (saving) return;
		saving = true;
		try {
			await saveArtifactContent(artifact.id, cardId, content);
		} catch {
			// silently fail — user can retry
		}
		saving = false;
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center gap-2 border-b border-border pb-3">
		<button
			class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			onclick={onback}
			aria-label="Back to artifacts list"
			tabindex="0"
		>
			<ArrowLeftIcon size={16} />
		</button>
		<h3 class="min-w-0 flex-1 truncate text-sm font-medium">
			{artifact.name}
		</h3>
		{#if saving}
			<span class="text-[11px] text-muted-foreground">Saving...</span>
		{/if}
	</div>

	<!-- Editor -->
	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-muted-foreground">Loading...</p>
		</div>
	{:else}
		<div class="grid flex-1 grid-cols-2 gap-4 pt-3" style="min-height: 300px;">
			<!-- Textarea -->
			<textarea
				class="w-full resize-none rounded-md border border-input bg-background p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				bind:value={content}
				oninput={handleInput}
				onblur={handleBlur}
				placeholder="Write markdown content..."
			></textarea>

			<!-- Preview -->
			<div
				class="overflow-y-auto rounded-md border border-border bg-muted/30 p-3 text-sm"
			>
				<div class="prose prose-sm dark:prose-invert max-w-none whitespace-pre-wrap">
					{content || 'Preview will appear here...'}
				</div>
			</div>
		</div>
	{/if}
</div>
