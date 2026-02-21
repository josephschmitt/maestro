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

	let {
		open = $bindable(false),
		runningCount = 0,
		onstopall,
		onkeeprunning,
		oncancel
	}: {
		open: boolean;
		runningCount: number;
		onstopall: () => void;
		onkeeprunning: () => void;
		oncancel: () => void;
	} = $props();
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Agents are running</DialogTitle>
			<DialogDescription>
				{runningCount} agent{runningCount === 1 ? ' is' : 's are'} still running. What would you like to do?
			</DialogDescription>
		</DialogHeader>
		<DialogFooter class="flex gap-2 sm:flex-col sm:space-x-0">
			<Button variant="destructive" onclick={onstopall}>
				Stop all agents
			</Button>
			<Button variant="outline" onclick={onkeeprunning}>
				Keep running in background
			</Button>
			<Button variant="ghost" onclick={oncancel}>
				Cancel
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
