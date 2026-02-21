<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button/index.js';

	interface CrashedAgent {
		workspace_id: string;
		project_id: string;
		card_id: string;
		session_id: string | null;
	}

	let {
		open = $bindable(false),
		crashedAgents = [],
		onresume,
		ondismiss
	}: {
		open: boolean;
		crashedAgents: CrashedAgent[];
		onresume: (workspaceId: string, projectId: string) => void;
		ondismiss: () => void;
	} = $props();
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Agent{crashedAgents.length === 1 ? '' : 's'} crashed</DialogTitle>
			<DialogDescription>
				{crashedAgents.length} agent session{crashedAgents.length === 1 ? '' : 's'} ended unexpectedly while the app was closed.
			</DialogDescription>
		</DialogHeader>
		<div class="flex flex-col gap-2 py-4">
			{#each crashedAgents as agent (agent.workspace_id)}
				<div class="flex items-center justify-between rounded-md border border-border px-3 py-2">
					<div class="flex flex-col">
						<span class="text-sm font-medium">Session {agent.workspace_id.slice(0, 8)}</span>
						{#if agent.session_id}
							<span class="text-xs text-muted-foreground">Resumable</span>
						{:else}
							<span class="text-xs text-muted-foreground">No session to resume</span>
						{/if}
					</div>
					{#if agent.session_id}
						<Button
							size="sm"
							variant="outline"
							onclick={() => onresume(agent.workspace_id, agent.project_id)}
						>
							Resume
						</Button>
					{/if}
				</div>
			{/each}
		</div>
		<DialogFooter>
			<Button variant="ghost" onclick={ondismiss}>Dismiss</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
