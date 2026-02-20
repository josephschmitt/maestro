import { invoke } from '@tauri-apps/api/core';

function isTauriAvailable(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauriAvailable()) {
		const { dispatchMockCommand } = await import('./mock/index.js');
		return dispatchMockCommand<T>(command, args);
	}
	return invoke<T>(command, args);
}
