<script lang="ts">
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import { getRandomVerb, getVerbForContext, type VerbContext } from '$lib/utils/spinner-verbs.js';
	import { onDestroy } from 'svelte';

	const props: { context?: VerbContext } = $props();

	const pickVerb = () => (props.context ? getVerbForContext(props.context) : getRandomVerb());

	let verb = $state(pickVerb());
	let visible = $state(true);

	const interval = setInterval(() => {
		visible = false;
		setTimeout(() => {
			verb = pickVerb();
			visible = true;
		}, 300);
	}, 3500);

	onDestroy(() => clearInterval(interval));
</script>

<div class="flex items-center gap-2" role="status" aria-label="Loading">
	<span class="spinner-icon text-muted-foreground">
		<SparklesIcon size={14} />
	</span>
	<span
		class="spinner-text text-sm text-muted-foreground transition-opacity duration-300"
		class:opacity-0={!visible}
	>
		{verb}
	</span>
	<span class="flex gap-0.5" aria-hidden="true">
		<span class="spinner-dot size-1 rounded-full bg-muted-foreground"></span>
		<span class="spinner-dot size-1 rounded-full bg-muted-foreground [animation-delay:150ms]"
		></span>
		<span class="spinner-dot size-1 rounded-full bg-muted-foreground [animation-delay:300ms]"
		></span>
	</span>
</div>

<style>
	.spinner-icon {
		animation: pulse 2s ease-in-out infinite;
	}

	.spinner-text {
		background: linear-gradient(
			90deg,
			var(--muted-foreground) 0%,
			var(--foreground) 50%,
			var(--muted-foreground) 100%
		);
		background-size: 200% 100%;
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		animation: shimmer 2s ease-in-out infinite;
	}

	.spinner-dot {
		animation: bounce 1.4s ease-in-out infinite;
	}

	@keyframes shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.5;
			transform: scale(0.9);
		}
	}

	@keyframes bounce {
		0%,
		80%,
		100% {
			transform: translateY(0);
		}
		40% {
			transform: translateY(-4px);
		}
	}
</style>
