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
		title = 'Confirm action',
		message = 'Are you sure you want to proceed?',
		confirmLabel = 'Confirm',
		cancelLabel = 'Cancel',
		variant = 'destructive' as 'destructive' | 'default',
		loading = false,
		onconfirm,
		oncancel
	}: {
		open: boolean;
		title?: string;
		message?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		variant?: 'destructive' | 'default';
		loading?: boolean;
		onconfirm: () => void;
		oncancel?: () => void;
	} = $props();

	function handleCancel() {
		open = false;
		oncancel?.();
	}

	function handleConfirm() {
		onconfirm();
	}
</script>

<Dialog bind:open>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>{title}</DialogTitle>
			<DialogDescription>{message}</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="outline" onclick={handleCancel} disabled={loading}>
				{cancelLabel}
			</Button>
			<Button {variant} onclick={handleConfirm} disabled={loading}>
				{#if loading}
					<span class="mr-2 inline-block size-3 animate-spin rounded-full border-2 border-current border-t-transparent"></span>
				{/if}
				{confirmLabel}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
