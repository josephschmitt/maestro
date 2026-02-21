<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronUpIcon from '@lucide/svelte/icons/chevron-up';

	const AVAILABLE_PROMPTS = [
		{ id: 'brainstorming', label: 'Brainstorming', description: 'Design-first requirements exploration' },
		{ id: 'implementation-planning', label: 'Implementation Planning', description: 'Work decomposition for delegation' },
		{ id: 'tdd', label: 'TDD', description: 'Test-driven development (RED-GREEN-REFACTOR)' },
		{ id: 'systematic-debugging', label: 'Systematic Debugging', description: '4-phase root cause analysis' },
		{ id: 'verification', label: 'Verification', description: 'Evidence-based verification before completion' },
		{ id: 'code-review', label: 'Code Review', description: 'Structured code review response' }
	] as const;

	interface Props {
		statusPrompts: string[];
		onchange: (prompts: string[]) => void;
	}

	let { statusPrompts, onchange }: Props = $props();
	let expanded = $state(false);

	function getLabel(id: string): string {
		return AVAILABLE_PROMPTS.find((p) => p.id === id)?.label ?? id;
	}

	function toggle(promptId: string) {
		const next = statusPrompts.includes(promptId)
			? statusPrompts.filter((p) => p !== promptId)
			: [...statusPrompts, promptId];
		onchange(next);
	}
</script>

<div class="w-full">
	<button
		type="button"
		class="flex w-full items-center gap-1 rounded px-1 py-0.5 text-left text-xs text-muted-foreground hover:bg-accent"
		onclick={() => (expanded = !expanded)}
	>
		{#if statusPrompts.length === 0}
			<span class="italic">No prompts</span>
		{:else}
			<span class="flex flex-wrap gap-1">
				{#each statusPrompts as promptId (promptId)}
					<span class="rounded bg-muted px-1.5 py-0.5 text-xs">{getLabel(promptId)}</span>
				{/each}
			</span>
		{/if}
		<span class="ml-auto shrink-0">
			{#if expanded}
				<ChevronUpIcon class="size-3" />
			{:else}
				<ChevronDownIcon class="size-3" />
			{/if}
		</span>
	</button>

	{#if expanded}
		<div class="mt-1 space-y-0.5 rounded-md border border-border bg-card p-2">
			{#each AVAILABLE_PROMPTS as prompt (prompt.id)}
				<Button
					variant="ghost"
					size="sm"
					class="h-auto w-full justify-start gap-2 px-2 py-1.5 text-left font-normal"
					onclick={() => toggle(prompt.id)}
				>
					<span
						class="flex size-4 shrink-0 items-center justify-center rounded border border-border {statusPrompts.includes(prompt.id) ? 'border-primary bg-primary text-primary-foreground' : ''}"
					>
						{#if statusPrompts.includes(prompt.id)}
							<CheckIcon class="size-3" />
						{/if}
					</span>
					<span class="flex flex-col gap-0">
						<span class="text-sm">{prompt.label}</span>
						<span class="text-xs text-muted-foreground">{prompt.description}</span>
					</span>
				</Button>
			{/each}
		</div>
	{/if}
</div>
