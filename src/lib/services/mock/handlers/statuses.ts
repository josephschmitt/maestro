import type { Status } from '$lib/types/index.js';
import type { StatusGroup } from '$lib/types/status.js';
import { getStore, newId, nowISO } from '../store.js';

export function list_statuses(args: Record<string, unknown>): Status[] {
	const store = getStore();
	return store.statuses
		.filter((s) => s.project_id === args.projectId)
		.sort((a, b) => a.sort_order - b.sort_order);
}

export function create_status(args: Record<string, unknown>): Status {
	const store = getStore();
	const projectStatuses = store.statuses.filter((s) => s.project_id === args.projectId);
	const maxOrder = projectStatuses.reduce((max, s) => Math.max(max, s.sort_order), -1);

	const status: Status = {
		id: newId(),
		project_id: args.projectId as string,
		group: args.group as StatusGroup,
		name: args.name as string,
		sort_order: maxOrder + 1,
		is_default: (args.isDefault as boolean) ?? false,
		created_at: nowISO()
	};
	store.statuses.push(status);
	return status;
}

export function update_status(args: Record<string, unknown>): Status {
	const store = getStore();
	const status = store.statuses.find(
		(s) => s.id === args.id && s.project_id === args.projectId
	);
	if (!status) throw new Error(`Status not found: ${args.id}`);
	if (args.name !== undefined) status.name = args.name as string;
	if (args.isDefault !== undefined) status.is_default = args.isDefault as boolean;
	return status;
}

export function delete_status(args: Record<string, unknown>): void {
	const store = getStore();
	store.statuses = store.statuses.filter((s) => s.id !== args.id);
}

export function reorder_statuses(args: Record<string, unknown>): Status[] {
	const store = getStore();
	const statusIds = args.statusIds as string[];
	statusIds.forEach((id, index) => {
		const status = store.statuses.find((s) => s.id === id);
		if (status) status.sort_order = index;
	});
	return list_statuses(args);
}
