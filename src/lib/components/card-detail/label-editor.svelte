<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';

	let {
		labels,
		onchange
	}: {
		labels: string[];
		onchange: (labels: string[]) => void;
	} = $props();

	let newLabel = $state('');

	function addLabel() {
		const trimmed = newLabel.trim();
		if (trimmed && !labels.includes(trimmed)) {
			onchange([...labels, trimmed]);
		}
		newLabel = '';
	}

	function removeLabel(label: string) {
		onchange(labels.filter((l) => l !== label));
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addLabel();
		}
	}

	function labelColor(label: string): string {
		let hash = 0;
		for (let i = 0; i < label.length; i++) {
			hash = label.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = Math.abs(hash) % 360;
		return `hsl(${hue}, 60%, 85%)`;
	}

	function labelTextColor(label: string): string {
		let hash = 0;
		for (let i = 0; i < label.length; i++) {
			hash = label.charCodeAt(i) + ((hash << 5) - hash);
		}
		const hue = Math.abs(hash) % 360;
		return `hsl(${hue}, 50%, 30%)`;
	}
</script>

<div class="flex flex-col gap-2">
	{#if labels.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each labels as label (label)}
				<span
					class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium"
					style="background-color: {labelColor(label)}; color: {labelTextColor(label)}"
				>
					{label}
					<button
						class="inline-flex items-center rounded-full p-0.5 opacity-60 hover:opacity-100"
						onclick={() => removeLabel(label)}
						aria-label="Remove label {label}"
					>
						<XIcon size={10} />
					</button>
				</span>
			{/each}
		</div>
	{/if}
	<input
		type="text"
		bind:value={newLabel}
		onkeydown={handleKeydown}
		placeholder="Add label..."
		class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
	/>
</div>
