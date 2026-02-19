import type { StatusGroup } from './status.js';

export interface CardWithStatus {
	id: string;
	project_id: string;
	parent_id: string | null;
	status_id: string;
	title: string;
	description: string;
	labels: string[];
	sort_order: number;
	created_at: string;
	updated_at: string;
	status_name: string;
	status_group: StatusGroup;
}

export interface CardProgress {
	completed: number;
	total: number;
}
