<script lang="ts">
	import {
		conversations,
		messages,
		loadConversations,
		loadMessages,
		sendMessage
	} from '$lib/stores/conversations.js';
	import MessageBubble from '$lib/components/card-detail/message-bubble.svelte';
	import MessageInput from '$lib/components/card-detail/message-input.svelte';

	let { cardId }: { cardId: string } = $props();

	let messageContainer: HTMLDivElement | undefined = $state();

	let reviewConversation = $derived(
		$conversations.find((c) => c.agent_type === 'review') ?? null
	);

	$effect(() => {
		loadConversations(cardId);
	});

	$effect(() => {
		if (reviewConversation) {
			loadMessages(reviewConversation.id);
		} else {
			messages.set([]);
		}
	});

	$effect(() => {
		if ($messages && messageContainer) {
			requestAnimationFrame(() => {
				if (messageContainer) {
					messageContainer.scrollTop = messageContainer.scrollHeight;
				}
			});
		}
	});

	async function handleSend(content: string) {
		if (!reviewConversation) return;
		await sendMessage(reviewConversation.id, content);
	}
</script>

<div class="flex flex-col border-t border-border">
	<div class="px-3 py-2 border-b border-border">
		<span class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
			Review Comments
		</span>
	</div>

	<div bind:this={messageContainer} class="max-h-[200px] overflow-y-auto px-3 py-2">
		{#if $messages.length > 0}
			<div class="flex flex-col gap-2">
				{#each $messages as msg (msg.id)}
					<MessageBubble message={msg} />
				{/each}
			</div>
		{:else}
			<p class="py-2 text-center text-xs text-muted-foreground">
				No review comments yet.
			</p>
		{/if}
	</div>

	{#if reviewConversation}
		<MessageInput onsend={handleSend} />
	{/if}
</div>
