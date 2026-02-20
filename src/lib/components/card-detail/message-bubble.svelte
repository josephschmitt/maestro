<script lang="ts">
	import type { ConversationMessage } from '$lib/types/index.js';
	import { marked } from 'marked';

	let {
		message
	}: {
		message: ConversationMessage;
	} = $props();

	let isUser = $derived(message.role === 'user');

	let renderedContent = $derived(marked.parse(message.content, { async: false }) as string);

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

<div class="flex {isUser ? 'justify-end' : 'justify-start'}">
	<div
		class="max-w-[80%] rounded-lg px-3 py-2 text-sm {isUser
			? 'bg-primary text-primary-foreground'
			: 'bg-muted text-foreground'}"
	>
		<div class="prose prose-sm max-w-none {isUser ? 'prose-invert' : ''} [&>*:first-child]:mt-0 [&>*:last-child]:mb-0">
			{@html renderedContent}
		</div>
		<div
			class="mt-1 text-[10px] {isUser
				? 'text-primary-foreground/70'
				: 'text-muted-foreground'}"
		>
			{formatRelativeTime(message.timestamp)}
		</div>
	</div>
</div>
