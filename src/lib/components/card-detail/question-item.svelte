<script lang="ts">
	import type { OpenQuestion } from '$lib/types/index.js';
	import CircleIcon from '@lucide/svelte/icons/circle';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	let {
		question,
		onresolve,
		onunresolve,
		ondelete
	}: {
		question: OpenQuestion;
		onresolve: (id: string) => void;
		onunresolve: (id: string) => void;
		ondelete: (id: string) => void;
	} = $props();

	let isResolved = $derived(question.resolved_at !== null);
</script>

<div
	class="group flex items-start gap-2 rounded-md px-2 py-1.5 {isResolved ? 'opacity-60' : ''}"
>
	<button
		class="mt-0.5 shrink-0 text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none rounded-sm"
		onclick={() => isResolved ? onunresolve(question.id) : onresolve(question.id)}
		aria-label={isResolved ? 'Mark as unresolved' : 'Mark as resolved'}
		tabindex="0"
	>
		{#if isResolved}
			<CircleCheckIcon size={16} class="text-green-500" />
		{:else}
			<CircleIcon size={16} />
		{/if}
	</button>

	<div class="min-w-0 flex-1">
		<p class="text-sm {isResolved ? 'line-through text-muted-foreground' : 'text-foreground'}">
			{question.question}
		</p>
		<div class="mt-0.5 flex items-center gap-1.5">
			<span
				class="inline-flex rounded-full px-1.5 py-0.5 text-[10px] font-medium leading-tight {question.source === 'agent'
					? 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
					: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'}"
			>
				{question.source}
			</span>
			{#if question.resolution}
				<span class="text-[11px] text-muted-foreground italic">
					{question.resolution}
				</span>
			{/if}
		</div>
	</div>

	<button
		class="shrink-0 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none rounded-sm"
		onclick={() => ondelete(question.id)}
		aria-label="Delete question"
		tabindex="0"
	>
		<Trash2Icon size={14} />
	</button>
</div>
