<script lang="ts">
	import type { Status, StatusGroup } from '$lib/types/index.js';
	import { STATUS_GROUPS } from '$lib/types/index.js';
	import * as Select from '$lib/components/ui/select/index.js';

	let {
		statuses,
		currentStatusId,
		onchange
	}: {
		statuses: Status[];
		currentStatusId: string;
		onchange: (statusId: string) => void;
	} = $props();

	let statusesByGroup = $derived(() => {
		const map = new Map<StatusGroup, Status[]>();
		for (const group of STATUS_GROUPS) {
			const groupStatuses = statuses.filter((s) => s.group === group);
			if (groupStatuses.length > 0) {
				map.set(group, groupStatuses);
			}
		}
		return map;
	});

	let currentStatus = $derived(statuses.find((s) => s.id === currentStatusId));

	function handleValueChange(val: string | undefined) {
		if (val && val !== currentStatusId) {
			onchange(val);
		}
	}
</script>

<Select.Root type="single" value={currentStatusId} onValueChange={handleValueChange}>
	<Select.Trigger size="sm" class="h-7 text-xs">
		{currentStatus?.name ?? 'Select status'}
	</Select.Trigger>
	<Select.Content>
		{#each STATUS_GROUPS as group (group)}
			{@const groupStatuses = statusesByGroup().get(group)}
			{#if groupStatuses && groupStatuses.length > 0}
				<Select.Group>
					<Select.GroupHeading>{group}</Select.GroupHeading>
					{#each groupStatuses as status (status.id)}
						<Select.Item value={status.id} label={status.name} />
					{/each}
				</Select.Group>
			{/if}
		{/each}
	</Select.Content>
</Select.Root>
