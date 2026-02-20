<script lang="ts">
	import SendIcon from '@lucide/svelte/icons/send';

	let {
		onsend
	}: {
		onsend: (content: string) => void;
	} = $props();

	let text = $state('');
	let textarea: HTMLTextAreaElement | undefined = $state();

	function handleSend() {
		const trimmed = text.trim();
		if (!trimmed) return;
		onsend(trimmed);
		text = '';
		if (textarea) {
			textarea.style.height = 'auto';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			handleSend();
		}
	}

	function handleInput() {
		if (!textarea) return;
		textarea.style.height = 'auto';
		textarea.style.height = `${textarea.scrollHeight}px`;
	}
</script>

<div class="flex gap-2 border-t border-border p-3">
	<textarea
		bind:this={textarea}
		bind:value={text}
		onkeydown={handleKeydown}
		oninput={handleInput}
		class="flex-1 resize-none rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
		placeholder="Type a message... (Cmd+Enter to send)"
		rows="2"
	></textarea>
	<button
		class="inline-flex shrink-0 items-center justify-center self-end rounded-md bg-primary p-2 text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
		onclick={handleSend}
		disabled={!text.trim()}
		aria-label="Send message"
	>
		<SendIcon size={16} />
	</button>
</div>
