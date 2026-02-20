<script lang="ts">
	import type { OpenQuestion } from '$lib/types/index.js';
	import {
		questions,
		unresolvedQuestions,
		resolvedQuestions,
		loadQuestions,
		addQuestion,
		resolveQuestion,
		unresolveQuestion,
		removeQuestion
	} from '$lib/stores/questions.js';
	import QuestionItem from '../question-item.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let {
		cardId
	}: {
		cardId: string;
	} = $props();

	let newQuestionText = $state('');
	let showResolved = $state(false);

	$effect(() => {
		loadQuestions(cardId);
	});

	async function handleAddQuestion() {
		const text = newQuestionText.trim();
		if (!text) return;
		await addQuestion(cardId, text, 'user');
		newQuestionText = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleAddQuestion();
		}
	}

	async function handleResolve(id: string) {
		await resolveQuestion(id, cardId, 'user');
	}

	async function handleUnresolve(id: string) {
		await unresolveQuestion(id, cardId);
	}

	async function handleDelete(id: string) {
		await removeQuestion(id, cardId);
	}
</script>

<div class="flex flex-col gap-3">
	<!-- Add question form -->
	<div class="flex gap-2">
		<input
			type="text"
			class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			placeholder="Add an open question..."
			bind:value={newQuestionText}
			onkeydown={handleKeydown}
		/>
		<button
			class="inline-flex shrink-0 items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
			onclick={handleAddQuestion}
			disabled={!newQuestionText.trim()}
			aria-label="Add question"
		>
			<PlusIcon size={14} class="mr-1" />
			Add
		</button>
	</div>

	<!-- Unresolved questions -->
	{#if $unresolvedQuestions.length > 0}
		<div class="flex flex-col gap-0.5">
			{#each $unresolvedQuestions as q (q.id)}
				<QuestionItem
					question={q}
					onresolve={handleResolve}
					onunresolve={handleUnresolve}
					ondelete={handleDelete}
				/>
			{/each}
		</div>
	{:else if $resolvedQuestions.length === 0}
		<p class="py-4 text-center text-sm text-muted-foreground">
			No open questions yet.
		</p>
	{/if}

	<!-- Resolved questions (collapsed by default) -->
	{#if $resolvedQuestions.length > 0}
		<div>
			<button
				class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none rounded-sm"
				onclick={() => (showResolved = !showResolved)}
				aria-expanded={showResolved}
				tabindex="0"
			>
				{#if showResolved}
					<ChevronDownIcon size={12} />
				{:else}
					<ChevronRightIcon size={12} />
				{/if}
				{$resolvedQuestions.length} resolved
			</button>

			{#if showResolved}
				<div class="mt-1 flex flex-col gap-0.5">
					{#each $resolvedQuestions as q (q.id)}
						<QuestionItem
							question={q}
							onresolve={handleResolve}
							onunresolve={handleUnresolve}
							ondelete={handleDelete}
						/>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
