import type { GlobalConfigResponse, ResolvedAgentConfigResponse, HttpServerConfigResponse, HttpServerConfigUpdate } from '$lib/types/index.js';
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

export async function getHttpServerConfig(): Promise<HttpServerConfigResponse> {
	return tauriInvoke<HttpServerConfigResponse>('get_http_server_config');
}

export async function updateHttpServerConfig(update: HttpServerConfigUpdate): Promise<HttpServerConfigResponse> {
	return tauriInvoke<HttpServerConfigResponse>('update_http_server_config', { update });
}

export async function regenerateAuthToken(): Promise<string> {
	return tauriInvoke<string>('regenerate_auth_token');
}

export async function getLocalIp(): Promise<string> {
	return tauriInvoke<string>('get_local_ip');
}
