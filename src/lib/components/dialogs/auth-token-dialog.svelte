<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';

	let {
		open = $bindable(false),
		onconfirm
	}: {
		open: boolean;
		onconfirm: (token: string) => void;
	} = $props();

	let token = $state('');

	$effect(() => {
		if (open) {
			token = '';
		}
	});

	function handleConfirm() {
		const trimmed = token.trim();
		if (!trimmed) return;
		open = false;
		onconfirm(trimmed);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleConfirm();
		}
	}
</script>

<Dialog bind:open>
	<DialogContent
		showCloseButton={false}
		onInteractOutside={(e) => e.preventDefault()}
		onEscapeKeydown={(e) => e.preventDefault()}
	>
		<DialogHeader>
			<DialogTitle>Authentication required</DialogTitle>
			<DialogDescription>
				Enter the auth token to connect to the Maestro server. You can find this token in the server
				console output when it starts.
			</DialogDescription>
		</DialogHeader>
		<div class="py-4">
			<label for="auth-token" class="mb-1.5 flex items-center gap-2 text-sm font-medium">
				<KeyRoundIcon class="size-4" />
				Auth token
			</label>
			<Input
				id="auth-token"
				bind:value={token}
				onkeydown={handleKeydown}
				type="password"
				placeholder="Enter token..."
				class="font-mono text-sm"
				autofocus
			/>
		</div>
		<DialogFooter>
			<Button onclick={handleConfirm} disabled={!token.trim()}>Connect</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
