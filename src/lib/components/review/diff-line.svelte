<script lang="ts">
	import type { DiffLine } from '$lib/types/index.js';

	let { line }: { line: DiffLine } = $props();

	let bgClass = $derived(
		line.line_type === 'added'
			? 'bg-green-950/40'
			: line.line_type === 'removed'
				? 'bg-red-950/40'
				: ''
	);

	let textClass = $derived(
		line.line_type === 'added'
			? 'text-green-400'
			: line.line_type === 'removed'
				? 'text-red-400'
				: 'text-muted-foreground'
	);

	let prefix = $derived(
		line.line_type === 'added' ? '+' : line.line_type === 'removed' ? '-' : ' '
	);
</script>

<div class="flex font-mono text-xs leading-5 {bgClass}">
	<span class="w-10 shrink-0 select-none text-right text-muted-foreground/50 pr-1">
		{line.old_line ?? ''}
	</span>
	<span class="w-10 shrink-0 select-none text-right text-muted-foreground/50 pr-1">
		{line.new_line ?? ''}
	</span>
	<span class="w-4 shrink-0 select-none text-center {textClass}">{prefix}</span>
	<span class="flex-1 whitespace-pre-wrap break-all {textClass}">{line.content}</span>
</div>
