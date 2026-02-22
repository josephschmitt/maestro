import type { GlobalConfigResponse, ResolvedAgentConfigResponse, HttpServerConfigResponse } from '$lib/types/index.js';
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
