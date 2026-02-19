<script lang="ts">
	import StatusManager from '$lib/components/status-manager.svelte';
	import { currentProject } from '$lib/stores/project.js';
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex h-14 items-center border-b border-border px-6">
		<h1 class="text-lg font-semibold">
			{#if $currentProject}
				{$currentProject.name} — Settings
			{:else}
				Settings
			{/if}
		</h1>
	</div>

	<div class="flex-1 overflow-y-auto p-6">
		<div class="mx-auto max-w-2xl">
			<section>
				<h2 class="mb-4 text-base font-semibold">Status Management</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Manage the statuses available in each group. Cards move through these statuses on the
					board.
				</p>
				<StatusManager />
			</section>
		</div>
	</div>
</div>
