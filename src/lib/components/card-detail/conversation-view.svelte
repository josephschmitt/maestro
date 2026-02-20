<script lang="ts">
	import type { Conversation } from '$lib/types/index.js';
	import {
		messages,
		loadMessages,
		sendMessage
	} from '$lib/stores/conversations.js';
	import MessageBubble from './message-bubble.svelte';
	import MessageInput from './message-input.svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';

	let {
		conversation,
		onback
	}: {
		conversation: Conversation;
		onback: () => void;
	} = $props();

	let messageContainer: HTMLDivElement | undefined = $state();

	$effect(() => {
		loadMessages(conversation.id);
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
		await sendMessage(conversation.id, content);
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center gap-2 border-b border-border px-3 py-2">
		<button
			class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			onclick={onback}
			aria-label="Back to conversation list"
		>
			<ArrowLeftIcon size={16} />
		</button>
		<div class="flex flex-col">
			<span class="text-sm font-medium">{conversation.agent_type}</span>
			<span class="text-xs text-muted-foreground">
				{new Date(conversation.started_at).toLocaleDateString()}
			</span>
		</div>
	</div>

	<!-- Messages -->
	<div bind:this={messageContainer} class="flex-1 overflow-y-auto px-3 py-3">
		{#if $messages.length > 0}
			<div class="flex flex-col gap-3">
				{#each $messages as msg (msg.id)}
					<MessageBubble message={msg} />
				{/each}
			</div>
		{:else}
			<div class="flex h-full items-center justify-center">
				<p class="text-sm text-muted-foreground">No messages yet. Start the conversation.</p>
			</div>
		{/if}
	</div>

	<!-- Input -->
	<MessageInput onsend={handleSend} />
</div>
