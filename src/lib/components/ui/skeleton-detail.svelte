<script lang="ts">
	import Skeleton from './skeleton.svelte';
</script>

<div class="flex flex-col gap-4 p-4">
	<div class="flex items-start justify-between">
		<div class="flex-1">
			<Skeleton class="mb-2 h-6 w-48" />
			<Skeleton class="h-4 w-32" />
		</div>
		<Skeleton class="size-8 rounded" />
	</div>

	<div class="space-y-2">
		<Skeleton class="h-4 w-full" />
		<Skeleton class="h-4 w-5/6" />
		<Skeleton class="h-4 w-4/6" />
	</div>

	<div class="flex gap-2 border-b border-border pb-2">
		<Skeleton class="h-8 w-20 rounded-md" />
		<Skeleton class="h-8 w-24 rounded-md" />
		<Skeleton class="h-8 w-16 rounded-md" />
	</div>

	<div class="space-y-3">
		<Skeleton class="h-10 w-full rounded-md" />
		<div class="space-y-2">
			<Skeleton class="h-12 w-full rounded-md" />
			<Skeleton class="h-12 w-full rounded-md" />
		</div>
	</div>
</div>
