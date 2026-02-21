<script lang="ts">
	import type { ChangedFile, FileChangeStatus } from '$lib/types/index.js';
	import FilePlusIcon from '@lucide/svelte/icons/file-plus';
	import FilePenIcon from '@lucide/svelte/icons/file-pen-line';
	import FileMinusIcon from '@lucide/svelte/icons/file-minus';

	let {
		files,
		selectedPath,
		onselect
	}: {
		files: ChangedFile[];
		selectedPath: string | null;
		onselect: (path: string) => void;
	} = $props();

	function statusIcon(status: FileChangeStatus) {
		switch (status) {
			case 'A':
				return FilePlusIcon;
			case 'M':
				return FilePenIcon;
			case 'D':
				return FileMinusIcon;
		}
	}

	function statusColor(status: FileChangeStatus): string {
		switch (status) {
			case 'A':
				return 'text-green-500';
			case 'M':
				return 'text-yellow-500';
			case 'D':
				return 'text-red-500';
		}
	}

	function fileName(path: string): string {
		return path.split('/').pop() ?? path;
	}

	function fileDir(path: string): string {
		const parts = path.split('/');
		if (parts.length <= 1) return '';
		return parts.slice(0, -1).join('/') + '/';
	}
</script>

<div class="flex flex-col gap-0.5">
	{#each files as file (file.path)}
		{@const Icon = statusIcon(file.status)}
		<button
			class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none {selectedPath === file.path ? 'bg-muted' : ''}"
			onclick={() => onselect(file.path)}
			aria-label="View diff for {file.path}"
			tabindex={0}
		>
			<Icon size={14} class="shrink-0 {statusColor(file.status)}" />
			<span class="truncate">
				{#if fileDir(file.path)}
					<span class="text-muted-foreground">{fileDir(file.path)}</span>
				{/if}
				<span class="font-medium">{fileName(file.path)}</span>
			</span>
		</button>
	{/each}

	{#if files.length === 0}
		<p class="py-4 text-center text-sm text-muted-foreground">No changed files found.</p>
	{/if}
</div>
