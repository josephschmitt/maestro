<script lang="ts">
	import type { ConversationMessage } from '$lib/types/index.js';
	import { renderMarkdown } from '$lib/utils/markdown.js';
	import { createStreamingRenderer } from '$lib/utils/streaming-renderer.js';
	import { mount, unmount } from 'svelte';
	import CodeBlock from '$lib/components/ui/code-block.svelte';

	let {
		message,
		streaming = false
	}: {
		message: ConversationMessage;
		streaming?: boolean;
	} = $props();

	let isUser = $derived(message.role === 'user');
	let contentEl: HTMLDivElement | undefined = $state();

	let renderer: ReturnType<typeof createStreamingRenderer> | null = $state(null);
	let streamHtml = $state('');

	$effect(() => {
		if (streaming) {
			const r = createStreamingRenderer();
			renderer = r;
			return () => {
				r.destroy();
				renderer = null;
			};
		}
	});

	let renderedContent = $derived.by(() => {
		if (streaming && renderer) {
			return streamHtml;
		}
		return renderMarkdown(message.content);
	});

	$effect(() => {
		if (streaming && renderer) {
			renderer.update(message.content);
			const unsub = renderer.subscribe((html) => {
				streamHtml = html;
			});
			return () => {
				unsub();
			};
		}
	});

	let mountedBlocks: Array<Record<string, unknown>> = [];

	$effect(() => {
		void renderedContent;

		if (!contentEl) return;

		for (const instance of mountedBlocks) {
			unmount(instance);
		}
		mountedBlocks = [];

		const wrappers = contentEl.querySelectorAll('.code-block-wrapper');
		for (const wrapper of wrappers) {
			const lang = wrapper.getAttribute('data-lang') ?? 'Text';
			const rawCode = wrapper.getAttribute('data-raw-code') ?? '';
			const codeEl = wrapper.querySelector('code');
			const highlightedHtml = codeEl?.innerHTML ?? '';

			const decodedCode = new DOMParser().parseFromString(rawCode, 'text/html').body
				.textContent ?? '';

			wrapper.innerHTML = '';

			const instance = mount(CodeBlock, {
				target: wrapper,
				props: {
					language: lang,
					code: decodedCode,
					highlightedHtml
				}
			});
			mountedBlocks.push(instance);
		}

		return () => {
			for (const instance of mountedBlocks) {
				unmount(instance);
			}
			mountedBlocks = [];
		};
	});

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
		<div
			bind:this={contentEl}
			class="prose prose-sm max-w-none {isUser
				? 'prose-invert'
				: ''} [&>*:first-child]:mt-0 [&>*:last-child]:mb-0"
		>
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
