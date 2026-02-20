<script lang="ts">
	import type { Artifact } from '$lib/types/index.js';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	let {
		artifact,
		onclick,
		ondelete
	}: {
		artifact: Artifact;
		onclick: (id: string) => void;
		ondelete: (id: string) => void;
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

<div class="group flex items-center gap-2 rounded-md px-2 py-1.5">
	<button
		class="flex min-w-0 flex-1 items-center gap-2 text-left focus-visible:rounded-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
		onclick={() => onclick(artifact.id)}
		tabindex="0"
	>
		<FileTextIcon size={16} class="shrink-0 text-muted-foreground" />
		<span class="min-w-0 flex-1 truncate text-sm text-foreground">
			{artifact.name}
		</span>
		<span
			class="inline-flex shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-medium leading-tight {artifact.created_by === 'agent'
				? 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
				: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'}"
		>
			{artifact.created_by}
		</span>
		<span class="shrink-0 text-[11px] text-muted-foreground">
			{formatRelativeTime(artifact.updated_at)}
		</span>
	</button>

	<button
		class="shrink-0 rounded-sm text-muted-foreground opacity-0 hover:text-destructive focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none group-hover:opacity-100"
		onclick={(e) => {
			e.stopPropagation();
			ondelete(artifact.id);
		}}
		aria-label="Delete artifact"
		tabindex="0"
	>
		<Trash2Icon size={14} />
	</button>
</div>
