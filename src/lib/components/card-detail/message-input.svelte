<script lang="ts">
	import SendIcon from '@lucide/svelte/icons/send';
	import PasteChip from './paste-chip.svelte';
	import { createInputHistory } from '$lib/utils/input-history.js';
	import { toasts } from '$lib/stores/toasts.js';

	const PASTE_THRESHOLD = 500;
	const SUPPORTED_FILE_EXTENSIONS = new Set([
		'.txt', '.md', '.ts', '.js', '.py', '.rs', '.json', '.yaml', '.yml', '.toml', '.csv'
	]);

	interface Chip {
		id: string;
		content: string;
		preview: string;
		charCount: number;
		filename?: string;
	}

	let {
		onsend
	}: {
		onsend: (content: string) => void;
	} = $props();

	let text = $state('');
	let chips = $state<Chip[]>([]);
	let textarea: HTMLTextAreaElement | undefined = $state();
	let dragOver = $state(false);
	let expandedChipId = $state<string | null>(null);

	const history = createInputHistory();
	let nextChipId = 0;

	const totalChars = $derived(text.length + chips.reduce((sum, c) => sum + c.charCount, 0));
	const hasContent = $derived(text.trim().length > 0 || chips.length > 0);

	function createChip(content: string, filename?: string): Chip {
		return {
			id: `chip-${nextChipId++}`,
			content,
			preview: content.slice(0, 50).replace(/\n/g, ' '),
			charCount: content.length,
			filename
		};
	}

	function removeChip(id: string) {
		chips = chips.filter((c) => c.id !== id);
		if (expandedChipId === id) expandedChipId = null;
	}

	function toggleChipExpand(id: string) {
		expandedChipId = expandedChipId === id ? null : id;
	}

	function buildMessageContent(): string {
		const parts: string[] = [];
		if (text.trim()) {
			parts.push(text.trim());
		}
		for (const chip of chips) {
			parts.push(`<pasted-content>\n${chip.content}\n</pasted-content>`);
		}
		return parts.join('\n\n');
	}

	function handleSend() {
		if (!hasContent) return;
		const content = buildMessageContent();
		history.add(content);
		onsend(content);
		text = '';
		chips = [];
		expandedChipId = null;
		history.reset();
		if (textarea) {
			textarea.style.height = 'auto';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			handleSend();
			return;
		}

		if (e.key === 'ArrowUp' && !e.metaKey && !e.ctrlKey && !e.shiftKey) {
			if (textarea && textarea.selectionStart === 0 && textarea.selectionEnd === 0) {
				const prev = history.prev(text);
				if (prev !== null) {
					e.preventDefault();
					text = prev;
				}
			}
		}

		if (e.key === 'ArrowDown' && !e.metaKey && !e.ctrlKey && !e.shiftKey) {
			if (textarea && textarea.selectionStart === text.length && textarea.selectionEnd === text.length) {
				const next = history.next();
				if (next !== null) {
					e.preventDefault();
					text = next;
				}
			}
		}
	}

	function handleInput() {
		if (!textarea) return;
		textarea.style.height = 'auto';
		textarea.style.height = `${textarea.scrollHeight}px`;
	}

	function handlePaste(e: ClipboardEvent) {
		const pastedText = e.clipboardData?.getData('text/plain');
		if (pastedText && pastedText.length > PASTE_THRESHOLD) {
			e.preventDefault();
			chips = [...chips, createChip(pastedText)];
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dragOver = true;
	}

	function handleDragLeave(e: DragEvent) {
		const target = e.currentTarget as HTMLElement;
		const related = e.relatedTarget as Node | null;
		if (related && target.contains(related)) return;
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;

		const files = e.dataTransfer?.files;
		if (!files || files.length === 0) return;

		for (const file of files) {
			const ext = '.' + file.name.split('.').pop()?.toLowerCase();
			if (!SUPPORTED_FILE_EXTENSIONS.has(ext)) {
				toasts.warning('Unsupported file type', `"${file.name}" is not a supported text file.`);
				continue;
			}

			const reader = new FileReader();
			reader.onload = () => {
				const content = reader.result as string;
				chips = [...chips, createChip(content, file.name)];
			};
			reader.readAsText(file);
		}
	}

	function formatNumber(n: number): string {
		return n.toLocaleString();
	}
</script>

<div
	class="border-t border-border p-3 transition-colors {dragOver ? 'border-primary bg-primary/5' : ''}"
	role="region"
	aria-label="Message input"
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
>
	{#if chips.length > 0}
		<div class="mb-2 flex flex-wrap gap-1.5">
			{#each chips as chip (chip.id)}
				<PasteChip
					preview={chip.preview}
					charCount={chip.charCount}
					filename={chip.filename}
					onremove={() => removeChip(chip.id)}
					onclick={() => toggleChipExpand(chip.id)}
				/>
			{/each}
		</div>
	{/if}

	{#if expandedChipId}
		{@const expandedChip = chips.find((c) => c.id === expandedChipId)}
		{#if expandedChip}
			<div class="mb-2 max-h-48 overflow-auto rounded-md border border-border bg-muted/30 p-2">
				<div class="mb-1 flex items-center justify-between">
					<span class="text-xs font-medium text-muted-foreground">
						{expandedChip.filename ?? 'Pasted content'} — {formatNumber(expandedChip.charCount)} chars
					</span>
					<button
						type="button"
						class="text-xs text-muted-foreground hover:text-foreground"
						onclick={() => (expandedChipId = null)}
					>
						Collapse
					</button>
				</div>
				<pre class="whitespace-pre-wrap text-xs text-foreground">{expandedChip.content}</pre>
			</div>
		{/if}
	{/if}

	<div class="flex gap-2">
		<textarea
			bind:this={textarea}
			bind:value={text}
			onkeydown={handleKeydown}
			oninput={handleInput}
			onpaste={handlePaste}
			class="flex-1 resize-none rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			placeholder="Type a message... (Cmd+Enter to send)"
			rows="2"
		></textarea>
		<button
			class="inline-flex shrink-0 items-center justify-center self-end rounded-md bg-primary p-2 text-primary-foreground hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
			onclick={handleSend}
			disabled={!hasContent}
			aria-label="Send message"
		>
			<SendIcon size={16} />
		</button>
	</div>

	{#if totalChars > 0}
		<div class="mt-1 flex justify-end gap-2 text-xs text-muted-foreground">
			<span>{formatNumber(totalChars)} chars</span>
			{#if totalChars >= 1000}
				<span>~{formatNumber(Math.ceil(totalChars / 4))} tokens</span>
			{/if}
		</div>
	{/if}
</div>
