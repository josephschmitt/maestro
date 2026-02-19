import { invoke } from '@tauri-apps/api/core';

function isTauriAvailable(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauriAvailable()) {
		throw new Error(
			`Tauri runtime not available. Run "npm run tauri dev" to start the full desktop app.`
		);
	}
	return invoke<T>(command, args);
}
