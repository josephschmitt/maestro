<script lang="ts">
	let {
		class: className = '',
		variant = 'default' as 'default' | 'text' | 'circle' | 'card' | 'button'
	}: {
		class?: string;
		variant?: 'default' | 'text' | 'circle' | 'card' | 'button';
	} = $props();

	const variantStyles = {
		default: 'h-4 w-full',
		text: 'h-3 w-3/4',
		circle: 'size-8 rounded-full',
		card: 'h-20 w-full rounded-lg',
		button: 'h-8 w-20 rounded-md',
	};
</script>

<div
	class="animate-pulse rounded bg-muted {variantStyles[variant]} {className}"
	aria-hidden="true"
></div>
