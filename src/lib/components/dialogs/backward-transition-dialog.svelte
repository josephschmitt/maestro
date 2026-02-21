<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import type { RunningAgentChoice } from '$lib/transitions/gates.js';

	let {
		open,
		cardTitle,
		onchoice,
		oncancel
	}: {
		open: boolean;
		cardTitle: string;
		onchoice: (choice: RunningAgentChoice) => void;
		oncancel: () => void;
	} = $props();
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => { if (!isOpen) oncancel(); }}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title class="flex items-center gap-2">
				<OctagonAlertIcon size={18} class="text-amber-500" />
				Running Agent
			</Dialog.Title>
			<Dialog.Description>
				<strong>{cardTitle}</strong> has an agent currently running.
				How would you like to proceed?
			</Dialog.Description>
		</Dialog.Header>

		<div class="flex flex-col gap-2 py-2">
			<button
				class="rounded-md border border-border px-3 py-3 text-left hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				onclick={() => onchoice('stop')}
			>
				<div class="text-sm font-medium">Stop agent and move</div>
				<div class="text-xs text-muted-foreground">
					The agent will be stopped and the workspace paused. You can resume later.
				</div>
			</button>
			<button
				class="rounded-md border border-border px-3 py-3 text-left hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
				onclick={() => onchoice('keep-running')}
			>
				<div class="text-sm font-medium">Keep running in background</div>
				<div class="text-xs text-muted-foreground">
					The agent continues running but the card moves to the new status.
				</div>
			</button>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={oncancel}>Cancel</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
