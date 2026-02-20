<script lang="ts">
	import type { Conversation } from '$lib/types/index.js';
	import MessageSquareIcon from '@lucide/svelte/icons/message-square';

	let {
		conversations,
		messageCounts,
		onselect
	}: {
		conversations: Conversation[];
		messageCounts: Map<string, number>;
		onselect: (id: string) => void;
	} = $props();

	function formatRelativeTime(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		if (diffMins < 1) return 'just now';
		if (diffMins < 60) return `${diffMins}m ago`;
		const diffHours = Math.floor(diffMins / 60);
		if (diffHours < 24) return `${diffHours}h ago`;
		const diffDays = Math.floor(diffHours / 24);
		return `${diffDays}d ago`;
	}
</script>

<div class="flex flex-col gap-0.5">
	{#each conversations as conversation (conversation.id)}
		<button
			class="group flex w-full items-center gap-3 rounded-md px-3 py-2 text-left hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			onclick={() => onselect(conversation.id)}
			tabindex="0"
		>
			<div class="flex shrink-0 items-center justify-center rounded-md bg-muted p-1.5 text-muted-foreground group-hover:bg-background">
				<MessageSquareIcon size={14} />
			</div>
			<div class="flex min-w-0 flex-1 flex-col">
				<span class="truncate text-sm font-medium">{conversation.agent_type}</span>
				<span class="text-xs text-muted-foreground">
					{formatRelativeTime(conversation.started_at)} · {messageCounts.get(conversation.id) ?? 0} messages
				</span>
			</div>
		</button>
	{/each}
</div>
