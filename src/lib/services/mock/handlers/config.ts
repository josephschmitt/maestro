import type {
	GlobalConfigResponse,
	ResolvedAgentConfigResponse,
	HttpServerConfigResponse,
	AgentProfileInput,
	StatusGroupConfigInput,
	StatusGroupDefaultsResponse
} from '$lib/types/index.js';
import { getStore } from '../store.js';

export function get_global_config(): GlobalConfigResponse {
	return { ...getStore().globalConfig };
}

export function update_global_config(args: Record<string, unknown>): GlobalConfigResponse {
	const store = getStore();
	const update = args.update as { storage_base_path?: string; default_agent?: string } | undefined;
	if (update) {
		if (update.storage_base_path !== undefined) {
			store.globalConfig.storage_base_path = update.storage_base_path;
		}
		if (update.default_agent !== undefined) {
			store.globalConfig.default_agent = update.default_agent;
		}
	}
	return { ...store.globalConfig };
}

export function set_last_project(args: Record<string, unknown>): void {
	getStore().globalConfig.last_project_id = args.projectId as string;
}

export function resolve_config(args: Record<string, unknown>): ResolvedAgentConfigResponse {
	const store = getStore();
	const projectConfig = args.projectAgentConfig as Record<string, unknown> | undefined;
	const statusGroup = (args.statusGroup as string)?.toLowerCase();

	const projectStatusConfig = (projectConfig?.status as Record<string, Record<string, string>>)?.[
		statusGroup
	];
	if (projectStatusConfig) {
		return {
			agent: projectStatusConfig.agent ?? store.globalConfig.default_agent,
			model: projectStatusConfig.model ?? null,
			instructions: projectStatusConfig.instructions ?? null
		};
	}

	const globalStatusConfig = store.statusGroupDefaults[statusGroup];
	if (globalStatusConfig) {
		return {
			agent: globalStatusConfig.agent ?? store.globalConfig.default_agent,
			model: globalStatusConfig.model ?? null,
			instructions: globalStatusConfig.instructions ?? null
		};
	}

	return {
		agent: (projectConfig?.agent as string) ?? store.globalConfig.default_agent,
		model: null,
		instructions: null
	};
}

export function create_agent_profile(args: Record<string, unknown>): GlobalConfigResponse {
	const store = getStore();
	const profile = args.profile as AgentProfileInput;
	if (!store.globalConfig.agents.find((a) => a.name === profile.name)) {
		store.globalConfig.agents.push({
			name: profile.name,
			binary: profile.binary,
			flags: profile.flags,
			custom_command: profile.custom_command,
			env_vars: profile.env_vars
		});
	}
	return { ...store.globalConfig };
}

export function update_agent_profile(args: Record<string, unknown>): GlobalConfigResponse {
	const store = getStore();
	const name = args.name as string;
	const profile = args.profile as AgentProfileInput;
	const index = store.globalConfig.agents.findIndex((a) => a.name === name);
	if (index !== -1) {
		store.globalConfig.agents[index] = {
			name: profile.name,
			binary: profile.binary,
			flags: profile.flags,
			custom_command: profile.custom_command,
			env_vars: profile.env_vars
		};
	}
	return { ...store.globalConfig };
}

export function delete_agent_profile(args: Record<string, unknown>): GlobalConfigResponse {
	const store = getStore();
	const name = args.name as string;
	store.globalConfig.agents = store.globalConfig.agents.filter((a) => a.name !== name);
	return { ...store.globalConfig };
}

export function get_status_group_defaults(): StatusGroupDefaultsResponse {
	return { status: { ...getStore().statusGroupDefaults } };
}

export function update_status_group_defaults(
	args: Record<string, unknown>
): StatusGroupDefaultsResponse {
	const store = getStore();
	const statusGroup = (args.statusGroup as string).toLowerCase();
	const groupConfig = args.groupConfig as StatusGroupConfigInput;
	store.statusGroupDefaults[statusGroup] = groupConfig;
	return { status: { ...store.statusGroupDefaults } };
}

let mockHttpConfig = {
	enabled: true,
	port: 3456,
	bind_address: '127.0.0.1',
	auth_token: 'mock-token-12345678',
	requires_auth: false,
	server_url: 'http://127.0.0.1:3456'
};

export function get_http_server_config(): HttpServerConfigResponse {
	return { ...mockHttpConfig };
}

export function update_http_server_config(args: Record<string, unknown>): HttpServerConfigResponse {
	const update = args.update as { enabled?: boolean; port?: number; bind_address?: string } | undefined;
	if (update) {
		if (update.port !== undefined) {
			mockHttpConfig.port = update.port;
		}
		if (update.bind_address !== undefined) {
			mockHttpConfig.bind_address = update.bind_address;
			mockHttpConfig.requires_auth = update.bind_address !== '127.0.0.1' && update.bind_address !== 'localhost';
		}
		const displayAddress = mockHttpConfig.bind_address === '0.0.0.0' ? '192.168.1.100' : mockHttpConfig.bind_address;
		mockHttpConfig.server_url = `http://${displayAddress}:${mockHttpConfig.port}`;
	}
	return { ...mockHttpConfig };
}

export function regenerate_auth_token(): string {
	const newToken = 'mock-token-' + Math.random().toString(36).substring(2, 10);
	mockHttpConfig.auth_token = newToken;
	return newToken;
}

export function get_local_ip(): string {
	return '192.168.1.100';
}
