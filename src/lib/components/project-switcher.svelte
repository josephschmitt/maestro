<script lang="ts">
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from '$lib/components/ui/select/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import { projects, currentProject, switchProject } from '$lib/stores/project.js';

	let { onCreateClick }: { onCreateClick: () => void } = $props();

	let selectedValue = $derived($currentProject?.id ?? '');

	function handleValueChange(value: string) {
		if (value && value !== $currentProject?.id) {
			switchProject(value);
		}
	}
</script>

<div class="flex flex-col gap-1">
	{#if $projects.length > 0}
		<Select type="single" value={selectedValue} onValueChange={handleValueChange}>
			<SelectTrigger class="w-full" size="sm">
				<span data-slot="select-value" class="truncate">
					{#if $currentProject}
						<FolderIcon class="size-3.5" />
						{$currentProject.name}
					{:else}
						Select project...
					{/if}
				</span>
			</SelectTrigger>
			<SelectContent>
				{#each $projects as project (project.id)}
					<SelectItem value={project.id} label={project.name} />
				{/each}
			</SelectContent>
		</Select>
	{/if}
	<Button variant="ghost" size="sm" class="w-full justify-start gap-2" onclick={onCreateClick}>
		<PlusIcon class="size-3.5" />
		New project
	</Button>
</div>
