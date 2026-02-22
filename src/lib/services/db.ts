import { invoke } from '@tauri-apps/api/core';

function isTauriAvailable(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function getHttpBaseUrl(): string | null {
	if (typeof window !== 'undefined' && (window as { __MAESTRO_HTTP_URL__?: string }).__MAESTRO_HTTP_URL__) {
		return (window as { __MAESTRO_HTTP_URL__?: string }).__MAESTRO_HTTP_URL__ as string;
	}
	return null;
}

function getAuthToken(): string | null {
	if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
		return localStorage.getItem('maestro_auth_token');
	}
	return null;
}

export function setAuthToken(token: string): void {
	if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
		localStorage.setItem('maestro_auth_token', token);
	}
}

export function clearAuthToken(): void {
	if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
		localStorage.removeItem('maestro_auth_token');
	}
}

export type TransportMode = 'tauri' | 'http' | 'mock';

export function getTransportMode(): TransportMode {
	if (isTauriAvailable()) return 'tauri';
	if (getHttpBaseUrl()) return 'http';
	return 'mock';
}

export function getWebSocketUrl(path: string): string {
	const httpUrl = getHttpBaseUrl();
	if (!httpUrl) {
		throw new Error('No HTTP base URL configured for WebSocket connection');
	}
	const wsUrl = httpUrl.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:');
	return `${wsUrl}${path}`;
}

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	const mode = getTransportMode();

	switch (mode) {
		case 'tauri':
			return invoke<T>(command, args);

		case 'http': {
			const baseUrl = getHttpBaseUrl()!;
			const token = getAuthToken();
			const response = await fetch(`${baseUrl}/api/${command}`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					...(token ? { Authorization: `Bearer ${token}` } : {})
				},
				body: JSON.stringify(args ?? {})
			});
			if (!response.ok) {
				throw new Error(await response.text());
			}
			return response.json();
		}

		case 'mock': {
			const { dispatchMockCommand } = await import('./mock/index.js');
			return dispatchMockCommand<T>(command, args);
		}
	}
}
