import type { GlobalConfigResponse, ResolvedAgentConfigResponse } from '$lib/types/index.js';
import { getStore } from '../store.js';

export function get_global_config(): GlobalConfigResponse {
	return getStore().globalConfig;
}

export function set_last_project(args: Record<string, unknown>): void {
	getStore().globalConfig.last_project_id = args.projectId as string;
}

export function resolve_config(args: Record<string, unknown>): ResolvedAgentConfigResponse {
	const store = getStore();
	const projectConfig = args.projectAgentConfig as Record<string, unknown> | undefined;
	return {
		agent: (projectConfig?.agent as string) ?? store.globalConfig.default_agent,
		model: (projectConfig?.model as string) ?? null,
		instructions: (projectConfig?.instructions as string) ?? null
	};
}
