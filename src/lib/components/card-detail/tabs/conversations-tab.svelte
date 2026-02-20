<script lang="ts">
	import {
		conversations,
		activeConversationId,
		activeConversation,
		messageCountByConversation,
		loadConversations,
		addConversation
	} from '$lib/stores/conversations.js';
	import ConversationList from '../conversation-list.svelte';
	import ConversationView from '../conversation-view.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';

	let {
		cardId
	}: {
		cardId: string;
	} = $props();

	$effect(() => {
		activeConversationId.set(null);
		loadConversations(cardId);
	});

	function handleSelect(id: string) {
		activeConversationId.set(id);
	}

	async function handleNewConversation() {
		await addConversation(cardId, 'manual');
	}
</script>

{#if $activeConversation}
	<div class="h-[400px]">
		<ConversationView
			conversation={$activeConversation}
			onback={() => activeConversationId.set(null)}
		/>
	</div>
{:else}
	<div class="flex flex-col gap-3">
		<button
			class="inline-flex w-full items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			onclick={handleNewConversation}
			aria-label="New conversation"
		>
			<PlusIcon size={14} class="mr-1" />
			New Conversation
		</button>

		{#if $conversations.length > 0}
			<ConversationList conversations={$conversations} messageCounts={$messageCountByConversation} onselect={handleSelect} />
		{:else}
			<p class="py-4 text-center text-sm text-muted-foreground">
				No conversations yet. Start one to chat with an agent.
			</p>
		{/if}
	</div>
{/if}
