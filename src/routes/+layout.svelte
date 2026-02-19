<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex h-screen w-screen overflow-hidden">
	<aside
		class="flex w-56 shrink-0 flex-col border-r border-border bg-sidebar text-sidebar-foreground"
	>
		<div class="flex h-14 items-center gap-2 border-b border-sidebar-border px-4">
			<span class="text-lg font-semibold">Maestro</span>
		</div>
		<nav class="flex-1 p-2">
			<p class="px-2 py-1.5 text-sm text-muted-foreground">No projects yet</p>
		</nav>
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden">
		{@render children()}
	</main>
</div>
