<script lang="ts">
	import type { Status } from '$lib/types/index.js';
	import {
		changedFiles,
		selectedFilePath,
		fileDiff,
		reviewCount,
		reviewLoading,
		reviewError,
		loadChangedFiles,
		selectFile,
		loadReviewCount,
		resetReviewState
	} from '$lib/stores/review.js';
	import FileTree from '$lib/components/review/file-tree.svelte';
	import DiffView from '$lib/components/review/diff-view.svelte';
	import ReviewActions from '$lib/components/review/review-actions.svelte';
	import ReviewConversation from '$lib/components/review/review-conversation.svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import LoaderIcon from '@lucide/svelte/icons/loader';

	let {
		cardId,
		cardTitle,
		cardDescription,
		statuses,
		onstatuschange
	}: {
		cardId: string;
		cardTitle: string;
		cardDescription: string;
		statuses: Status[];
		onstatuschange: () => void;
	} = $props();

	$effect(() => {
		resetReviewState();
		loadChangedFiles(cardId);
		loadReviewCount(cardId);
	});

	function handleFileSelect(path: string) {
		selectFile(cardId, path);
	}

	function handleBackToFileList() {
		selectedFilePath.set(null);
		fileDiff.set(null);
	}
</script>

<div class="flex flex-col gap-4">
	{#if $reviewLoading && $changedFiles.length === 0}
		<div class="flex items-center justify-center gap-2 py-8">
			<LoaderIcon size={16} class="animate-spin text-muted-foreground" />
			<span class="text-sm text-muted-foreground">Loading changes...</span>
		</div>
	{:else if $reviewError && $changedFiles.length === 0}
		<div class="py-4 text-center">
			<p class="text-sm text-muted-foreground">
				No worktree found for this card. Review is available after an agent has worked on the card in implementation mode.
			</p>
		</div>
	{:else if $selectedFilePath && $fileDiff}
		<!-- Diff View for selected file -->
		<div class="flex flex-col gap-2">
			<div class="flex items-center gap-2">
				<button
					class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					onclick={handleBackToFileList}
					aria-label="Back to file list"
				>
					<ArrowLeftIcon size={16} />
				</button>
				<span class="text-sm font-medium font-mono truncate">{$selectedFilePath}</span>
			</div>
			<div class="max-h-[400px] overflow-y-auto">
				<DiffView diff={$fileDiff} />
			</div>
		</div>
	{:else}
		<!-- File tree -->
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
					Changed Files ({$changedFiles.length})
				</span>
			</div>
			<FileTree
				files={$changedFiles}
				selectedPath={$selectedFilePath}
				onselect={handleFileSelect}
			/>
		</div>
	{/if}

	<ReviewConversation {cardId} />

	<ReviewActions
		{cardId}
		{cardTitle}
		cardDescription={cardDescription}
		{statuses}
		reviewCount={$reviewCount}
		{onstatuschange}
	/>
</div>
