<script lang="ts">
	import MarkdownEditor from '$lib/components/ui/markdown-editor.svelte';

	let {
		description,
		onchange
	}: {
		description: string;
		onchange: (description: string) => void;
	} = $props();

	let value = $state('');

	$effect(() => {
		value = description;
	});

	function handleChange(newValue: string) {
		if (newValue !== description) {
			onchange(newValue);
		}
	}
</script>

<div class="flex flex-col gap-2">
	<h3 class="text-sm font-medium text-foreground">Description</h3>
	<MarkdownEditor bind:value onchange={handleChange} />
</div>
