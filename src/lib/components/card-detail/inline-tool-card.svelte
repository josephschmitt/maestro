<script lang="ts">
	import type { ToolInvocation } from '$lib/types/index.js';
	import { getToolIcon } from '$lib/utils/tool-icons.js';
	import { renderMarkdown } from '$lib/utils/markdown.js';
	import { parseAnsi } from '$lib/utils/ansi-parser.js';
	import LoaderCircleIcon from '@lucide/svelte/icons/loader-circle';
	import CheckIcon from '@lucide/svelte/icons/check';
	import XIcon from '@lucide/svelte/icons/x';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let {
		invocation,
		expanded = false
	}: {
		invocation: ToolInvocation;
		expanded?: boolean;
	} = $props();

	let isExpanded = $state(expanded);
	let contentEl: HTMLDivElement | undefined = $state();
	let copied = $state(false);

	const toolIcon = $derived(getToolIcon(invocation.tool_name));

	const statusIcon = $derived.by(() => {
		switch (invocation.status) {
			case 'running':
				return LoaderCircleIcon;
			case 'completed':
				return CheckIcon;
			case 'failed':
				return XIcon;
		}
	});

	const statusColor = $derived.by(() => {
		switch (invocation.status) {
			case 'running':
				return 'text-blue-400';
			case 'completed':
				return 'text-green-400';
			case 'failed':
				return 'text-red-400';
		}
	});

	const durationLabel = $derived.by(() => {
		if (!invocation.duration_ms) return null;
		if (invocation.duration_ms < 1000) return `${invocation.duration_ms}ms`;
		return `${(invocation.duration_ms / 1000).toFixed(1)}s`;
	});

	const renderedOutput = $derived.by(() => {
		if (!isExpanded) return '';
		const content = invocation.error ?? invocation.output_full ?? invocation.output_preview ?? '';
		if (!content) return '';
		if (content.includes('\x1b[')) {
			return parseAnsi(content);
		}
		return renderMarkdown(content);
	});

	function toggle() {
		isExpanded = !isExpanded;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			toggle();
		}
	}

	async function copyOutput() {
		const text = invocation.error ?? invocation.output_full ?? invocation.output_preview ?? '';
		if (!text) return;
		await navigator.clipboard.writeText(text);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div
	class="group rounded-md border border-border/50 bg-muted/30 transition-colors hover:border-border"
	style="content-visibility: {isExpanded ? 'visible' : 'auto'}"
>
	<button
		class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm"
		onclick={toggle}
		onkeydown={handleKeydown}
		aria-expanded={isExpanded}
		role="button"
		tabindex={0}
	>
		<ChevronRightIcon
			class="size-3.5 shrink-0 text-muted-foreground transition-transform duration-150 {isExpanded ? 'rotate-90' : ''}"
		/>

		<svelte:component this={toolIcon.icon} class="size-4 shrink-0 {toolIcon.color}" />

		<span class="shrink-0 font-medium text-foreground">{invocation.tool_name}</span>

		<span class="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground">
			{invocation.input_summary}
		</span>

		{#if durationLabel}
			<span class="shrink-0 text-xs text-muted-foreground">{durationLabel}</span>
		{/if}

		<svelte:component
			this={statusIcon}
			class="size-3.5 shrink-0 {statusColor} {invocation.status === 'running' ? 'animate-spin' : ''}"
		/>
	</button>

	{#if isExpanded}
		<div class="border-t border-border/50">
			<div
				bind:this={contentEl}
				class="relative max-h-80 overflow-y-auto px-3 py-2 {invocation.error ? 'bg-red-500/5' : ''}"
			>
				{#if renderedOutput}
					<div class="prose prose-sm prose-invert max-w-none font-mono text-xs leading-relaxed">
						{@html renderedOutput}
					</div>
				{:else}
					<p class="text-xs italic text-muted-foreground">No output</p>
				{/if}

				{#if invocation.output_full || invocation.output_preview || invocation.error}
					<button
						class="absolute right-2 top-2 rounded-md p-1 text-muted-foreground opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100"
						onclick={copyOutput}
						title="Copy output"
					>
						{#if copied}
							<CheckIcon class="size-3.5 text-green-400" />
						{:else}
							<CopyIcon class="size-3.5" />
						{/if}
					</button>
				{/if}
			</div>
		</div>
	{/if}
</div>
