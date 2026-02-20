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
	import { showLinkDirectoryPrompt } from '$lib/stores/cards.js';
	import LinkDirectoryDialog from './link-directory-dialog.svelte';

	let linkDialogOpen = $state(false);

	function handleSkip() {
		showLinkDirectoryPrompt.set(false);
	}

	function handleBrowse() {
		showLinkDirectoryPrompt.set(false);
		linkDialogOpen = true;
	}
</script>

<Dialog
	open={$showLinkDirectoryPrompt}
	onOpenChange={(open) => {
		if (!open) showLinkDirectoryPrompt.set(false);
	}}
>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>No directories linked</DialogTitle>
			<DialogDescription>
				This project doesn't have any directories linked. Linked directories are used as
				working directories for agents. Would you like to link one now?
			</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="outline" onclick={handleSkip}>Skip</Button>
			<Button onclick={handleBrowse}>Browse</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<LinkDirectoryDialog bind:open={linkDialogOpen} />
