<script lang="ts">
	let {
		value = $bindable(''),
		onchange,
		placeholder = 'Add a description...'
	}: {
		value: string;
		onchange?: (value: string) => void;
		placeholder?: string;
	} = $props();

	let mode = $state<'edit' | 'preview'>('edit');

	function handleBlur() {
		onchange?.(value);
	}

	function simpleMarkdownToHtml(md: string): string {
		return md
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/^### (.+)$/gm, '<h3 class="text-base font-semibold mt-3 mb-1">$1</h3>')
			.replace(/^## (.+)$/gm, '<h2 class="text-lg font-semibold mt-3 mb-1">$1</h2>')
			.replace(/^# (.+)$/gm, '<h1 class="text-xl font-bold mt-3 mb-1">$1</h1>')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em>$1</em>')
			.replace(/`(.+?)`/g, '<code class="rounded bg-muted px-1 py-0.5 text-xs">$1</code>')
			.replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
			.replace(/\n/g, '<br>');
	}

	let previewHtml = $derived(simpleMarkdownToHtml(value || ''));
</script>

<div class="flex flex-col gap-2">
	<div class="flex gap-1">
		<button
			class="rounded px-2 py-1 text-xs font-medium {mode === 'edit'
				? 'bg-muted text-foreground'
				: 'text-muted-foreground hover:text-foreground'}"
			onclick={() => (mode = 'edit')}
		>
			Edit
		</button>
		<button
			class="rounded px-2 py-1 text-xs font-medium {mode === 'preview'
				? 'bg-muted text-foreground'
				: 'text-muted-foreground hover:text-foreground'}"
			onclick={() => (mode = 'preview')}
		>
			Preview
		</button>
	</div>

	{#if mode === 'edit'}
		<textarea
			bind:value
			onblur={handleBlur}
			{placeholder}
			class="min-h-[200px] w-full resize-y rounded-md border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
			rows="8"
		></textarea>
	{:else}
		<div
			class="min-h-[200px] rounded-md border border-input px-3 py-2 text-sm"
		>
			{#if value}
				{@html previewHtml}
			{:else}
				<p class="text-muted-foreground">Nothing to preview.</p>
			{/if}
		</div>
	{/if}
</div>
