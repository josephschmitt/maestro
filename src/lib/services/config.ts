import type {
	GlobalConfigResponse,
	GlobalConfigUpdate,
	ResolvedAgentConfigResponse,
	HttpServerConfigResponse,
	HttpServerConfigUpdate,
	AgentProfileInput,
	StatusGroupConfigInput,
	StatusGroupDefaultsResponse
} from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function getGlobalConfig(): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('get_global_config');
}

export async function updateGlobalConfig(
	update: GlobalConfigUpdate
): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('update_global_config', { update });
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

export async function createAgentProfile(
	profile: AgentProfileInput
): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('create_agent_profile', { profile });
}

export async function updateAgentProfile(
	name: string,
	profile: AgentProfileInput
): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('update_agent_profile', { name, profile });
}

export async function deleteAgentProfile(name: string): Promise<GlobalConfigResponse> {
	return tauriInvoke<GlobalConfigResponse>('delete_agent_profile', { name });
}

export async function getStatusGroupDefaults(): Promise<StatusGroupDefaultsResponse> {
	return tauriInvoke<StatusGroupDefaultsResponse>('get_status_group_defaults');
}

export async function updateStatusGroupDefaults(
	statusGroup: string,
	groupConfig: StatusGroupConfigInput
): Promise<StatusGroupDefaultsResponse> {
	return tauriInvoke<StatusGroupDefaultsResponse>('update_status_group_defaults', {
		statusGroup,
		groupConfig
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
