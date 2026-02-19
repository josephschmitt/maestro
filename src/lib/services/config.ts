import type { GlobalConfigResponse, ResolvedAgentConfigResponse } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function getGlobalConfig(): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('get_global_config');
}

export async function setLastProject(projectId: string): Promise<void> {
	return tauriInvoke<void>('set_last_project', { projectId });
}

export async function resolveConfig(
	projectAgentConfig: Record<string, unknown>,
	statusGroup: string
): Promise<ResolvedAgentConfigResponse> {
	return tauriInvoke<ResolvedAgentConfigResponse>('resolve_config', {
		projectAgentConfig,
		statusGroup
	});
}
