<script lang="ts">
	import { Check, Copy } from '@lucide/svelte';

	let {
		language,
		code,
		highlightedHtml
	}: {
		language: string;
		code: string;
		highlightedHtml: string;
	} = $props();

	let copied = $state(false);
	let copyTimeout: ReturnType<typeof setTimeout> | undefined;

	function handleCopy() {
		navigator.clipboard.writeText(code);
		copied = true;
		clearTimeout(copyTimeout);
		copyTimeout = setTimeout(() => {
			copied = false;
		}, 2000);
	}

	$effect(() => {
		return () => clearTimeout(copyTimeout);
	});
</script>

<div class="code-block group relative my-3 overflow-hidden rounded-md border border-border">
	<div
		class="flex items-center justify-between bg-muted/50 px-3 py-1 text-xs text-muted-foreground"
	>
		<span>{language}</span>
		<button
			onclick={handleCopy}
			class="flex items-center gap-1 rounded px-1.5 py-0.5 transition-colors hover:bg-accent hover:text-accent-foreground"
			aria-label={copied ? 'Copied' : 'Copy code'}
		>
			{#if copied}
				<Check class="h-3 w-3" />
				<span>Copied!</span>
			{:else}
				<Copy class="h-3 w-3" />
				<span>Copy</span>
			{/if}
		</button>
	</div>
	<div class="overflow-x-auto" style="content-visibility: auto;">
		<pre class="!my-0 !rounded-none !border-0"><code class="hljs">{@html highlightedHtml}</code></pre>
	</div>
</div>
