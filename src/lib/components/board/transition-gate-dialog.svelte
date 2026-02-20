<script lang="ts">
	import type { OpenQuestion } from '$lib/types/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import CircleHelpIcon from '@lucide/svelte/icons/circle-help';

	let {
		open,
		questions,
		cardTitle,
		onproceed,
		oncancel
	}: {
		open: boolean;
		questions: OpenQuestion[];
		cardTitle: string;
		onproceed: () => void;
		oncancel: () => void;
	} = $props();
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => { if (!isOpen) oncancel(); }}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title class="flex items-center gap-2">
				<CircleHelpIcon size={18} class="text-amber-500" />
				Unresolved Questions
			</Dialog.Title>
			<Dialog.Description>
				<strong>{cardTitle}</strong> has {questions.length} unresolved question{questions.length === 1 ? '' : 's'}.
				You may want to resolve them before starting work.
			</Dialog.Description>
		</Dialog.Header>

		<div class="max-h-48 overflow-y-auto">
			<ul class="flex flex-col gap-1.5">
				{#each questions as q (q.id)}
					<li class="flex items-start gap-2 rounded-md bg-muted/50 px-3 py-2 text-sm">
						<CircleHelpIcon size={14} class="mt-0.5 shrink-0 text-amber-500" />
						<span>{q.question}</span>
					</li>
				{/each}
			</ul>
		</div>

		<Dialog.Footer class="gap-2 sm:gap-0">
			<Button variant="outline" onclick={oncancel}>Cancel</Button>
			<Button variant="default" onclick={onproceed}>Proceed Anyway</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
