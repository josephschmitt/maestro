<script lang="ts">
	import type { Status } from '$lib/types/index.js';
	import { sendBack, approve, openPr } from '$lib/stores/review.js';
	import SendIcon from '@lucide/svelte/icons/send';
	import CheckIcon from '@lucide/svelte/icons/check';
	import GitPullRequestIcon from '@lucide/svelte/icons/git-pull-request';
	import UndoIcon from '@lucide/svelte/icons/undo-2';

	let {
		cardId,
		cardTitle,
		cardDescription,
		statuses,
		reviewCount,
		onstatuschange
	}: {
		cardId: string;
		cardTitle: string;
		cardDescription: string;
		statuses: Status[];
		reviewCount: number;
		onstatuschange: () => void;
	} = $props();

	let feedbackText = $state('');
	let sending = $state(false);
	let prCreating = $state(false);
	let prUrl = $state<string | null>(null);
	let error = $state<string | null>(null);

	let inProgressStatus = $derived(
		statuses.find((s) => s.group === 'Started' && s.is_default) ?? null
	);

	let completedStatus = $derived(
		statuses.find((s) => s.group === 'Completed' && s.is_default) ?? null
	);

	async function handleSendBack() {
		if (!feedbackText.trim() || !inProgressStatus) return;
		sending = true;
		error = null;
		try {
			await sendBack(cardId, feedbackText.trim(), inProgressStatus.id);
			feedbackText = '';
			onstatuschange();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			sending = false;
		}
	}

	async function handleApprove() {
		if (!completedStatus) return;
		sending = true;
		error = null;
		try {
			await approve(cardId, completedStatus.id);
			onstatuschange();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			sending = false;
		}
	}

	async function handleCreatePr() {
		prCreating = true;
		error = null;
		try {
			const url = await openPr(cardId, cardTitle, cardDescription);
			prUrl = url;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			prCreating = false;
		}
	}
</script>

<div class="flex flex-col gap-3 border-t border-border pt-3">
	{#if reviewCount > 0}
		<div class="text-xs text-muted-foreground">
			Review iteration: {reviewCount}
		</div>
	{/if}

	{#if error}
		<div class="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
			{error}
		</div>
	{/if}

	{#if prUrl}
		<div class="rounded-md bg-green-950/30 px-3 py-2 text-sm text-green-400">
			PR created: <a href={prUrl} target="_blank" rel="noopener noreferrer" class="underline">{prUrl}</a>
		</div>
	{/if}

	<!-- Send Back -->
	<div class="flex flex-col gap-2">
		<textarea
			bind:value={feedbackText}
			class="w-full resize-none rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			placeholder="Review feedback for the agent..."
			rows="3"
		></textarea>
		<button
			class="inline-flex w-full items-center justify-center gap-1.5 rounded-md border border-input bg-background px-3 py-1.5 text-sm font-medium hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
			onclick={handleSendBack}
			disabled={!feedbackText.trim() || sending || !inProgressStatus}
			aria-label="Send back with feedback"
		>
			<UndoIcon size={14} />
			Send Back
		</button>
	</div>

	<!-- Approve + Create PR -->
	<div class="flex gap-2">
		<button
			class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
			onclick={handleApprove}
			disabled={sending || !completedStatus}
			aria-label="Approve and complete"
		>
			<CheckIcon size={14} />
			Approve
		</button>
		<button
			class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-md border border-input bg-background px-3 py-1.5 text-sm font-medium hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
			onclick={handleCreatePr}
			disabled={prCreating}
			aria-label="Create pull request"
		>
			<GitPullRequestIcon size={14} />
			{prCreating ? 'Creating...' : 'Create PR'}
		</button>
	</div>
</div>
