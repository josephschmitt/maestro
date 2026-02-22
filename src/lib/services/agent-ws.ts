import { getTransportMode, getWebSocketUrl, type TransportMode } from './db.js';
import type { AgentOutputLine } from '$lib/stores/agent.js';

export interface AgentOutputEvent {
	stream: 'stdout' | 'stderr';
	line: string;
}

export interface AgentExitEvent {
	workspace_id: string;
	exit_code: number | null;
	status: string;
}

export interface AgentStreamConnection {
	sendInput: (text: string) => void;
	disconnect: () => void;
}

export function connectAgentStream(
	workspaceId: string,
	onOutput: (event: AgentOutputEvent) => void,
	onExit: (event: AgentExitEvent) => void
): AgentStreamConnection {
	const mode: TransportMode = getTransportMode();

	switch (mode) {
		case 'tauri':
			return connectTauriAgentStream(workspaceId, onOutput, onExit);

		case 'http':
			return connectHttpAgentStream(workspaceId, onOutput, onExit);

		case 'mock':
			return {
				sendInput: () => {
					// No-op in mock mode
				},
				disconnect: () => {
					// No-op in mock mode
				}
			};
	}
}

function connectTauriAgentStream(
	workspaceId: string,
	onOutput: (event: AgentOutputEvent) => void,
	onExit: (event: AgentExitEvent) => void
): AgentStreamConnection {
	let unlistenOutput: (() => void) | null = null;
	let unlistenExit: (() => void) | null = null;
	let disconnected = false;

	(async () => {
		try {
			const { listen } = await import('@tauri-apps/api/event');

			if (disconnected) return;

			unlistenOutput = await listen<AgentOutputLine>(
				`agent-output-${workspaceId}`,
				(event) => {
					onOutput(event.payload);
				}
			);

			if (disconnected) {
				unlistenOutput?.();
				return;
			}

			unlistenExit = await listen<AgentExitEvent>(
				`agent-exit-${workspaceId}`,
				(event) => {
					onExit(event.payload);
				}
			);
		} catch {
			// Not in Tauri environment - silently fail
		}
	})();

	return {
		sendInput: async (text: string) => {
			try {
				const { invoke } = await import('@tauri-apps/api/core');
				await invoke('send_agent_input', { workspaceId, text });
			} catch {
				// Not in Tauri environment
			}
		},
		disconnect: () => {
			disconnected = true;
			unlistenOutput?.();
			unlistenExit?.();
		}
	};
}

function connectHttpAgentStream(
	workspaceId: string,
	onOutput: (event: AgentOutputEvent) => void,
	onExit: (event: AgentExitEvent) => void
): AgentStreamConnection {
	let ws: WebSocket | null = null;
	let disconnected = false;

	try {
		const url = getWebSocketUrl(`/ws/agent/${workspaceId}`);
		ws = new WebSocket(url);

		ws.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				if (data.type === 'output') {
					onOutput(data.payload as AgentOutputEvent);
				} else if (data.type === 'exit') {
					onExit(data.payload as AgentExitEvent);
				}
			} catch (e) {
				console.error('Failed to parse agent WebSocket message:', e);
			}
		};

		ws.onerror = (error) => {
			console.error('Agent WebSocket error:', error);
		};

		ws.onclose = () => {
			if (!disconnected) {
				onExit({
					workspace_id: workspaceId,
					exit_code: null,
					status: 'failed'
				});
			}
		};
	} catch (e) {
		console.error('Failed to connect agent WebSocket:', e);
	}

	return {
		sendInput: (text: string) => {
			if (ws && ws.readyState === WebSocket.OPEN) {
				ws.send(JSON.stringify({ type: 'input', text }));
			}
		},
		disconnect: () => {
			disconnected = true;
			if (ws) {
				ws.close();
				ws = null;
			}
		}
	};
}
