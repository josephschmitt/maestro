<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import ProjectSwitcher from '$lib/components/project-switcher.svelte';
	import CreateProjectDialog from '$lib/components/dialogs/create-project-dialog.svelte';
	import { initializeProject, hasProject } from '$lib/stores/project.js';
	import { onMount } from 'svelte';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import { Button } from '$lib/components/ui/button/index.js';
	import { resolve } from '$app/paths';

	let { children } = $props();
	let createDialogOpen = $state(false);

	onMount(() => {
		initializeProject();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex h-screen w-screen overflow-hidden">
	<aside
		class="flex w-56 shrink-0 flex-col border-r border-border bg-sidebar text-sidebar-foreground"
	>
		<div
			data-tauri-drag-region
			class="flex h-14 items-center gap-2 border-b border-sidebar-border px-4 pt-5"
		>
			<span class="text-lg font-semibold">Maestro</span>
		</div>
		<nav class="flex-1 p-2">
			<ProjectSwitcher onCreateClick={() => (createDialogOpen = true)} />
		</nav>
		{#if $hasProject}
			<div class="space-y-0.5 border-t border-sidebar-border p-2">
				<a href={resolve('/board')}>
					<Button variant="ghost" size="sm" class="w-full justify-start gap-2">
						<LayoutDashboardIcon class="size-3.5" />
						Board
					</Button>
				</a>
				<a href={resolve('/settings')}>
					<Button variant="ghost" size="sm" class="w-full justify-start gap-2">
						<SettingsIcon class="size-3.5" />
						Settings
					</Button>
				</a>
			</div>
		{/if}
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden">
		<div data-tauri-drag-region class="h-8 w-full shrink-0"></div>
		{@render children()}
	</main>
</div>

<CreateProjectDialog bind:open={createDialogOpen} />
