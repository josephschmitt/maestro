import { getTransportMode, getWebSocketUrl, type TransportMode } from './db.js';

interface MaestroEvent {
	event_type: string;
	payload: unknown;
}

type EventCallback<T> = (payload: T) => void;

class WebSocketEventManager {
	private ws: WebSocket | null = null;
	private listeners = new Map<string, Set<EventCallback<unknown>>>();
	private reconnectAttempts = 0;
	private maxReconnectDelay = 30000;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private connecting = false;
	private connected = false;
	private hasConnectedBefore = false;

	async connect(): Promise<void> {
		if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
			return;
		}

		if (this.connecting) {
			return;
		}

		this.connecting = true;
		const isReconnect = this.hasConnectedBefore;

		try {
			const url = getWebSocketUrl('/ws/events');
			this.ws = new WebSocket(url);

			await new Promise<void>((resolve, reject) => {
				const ws = this.ws!;

				ws.onopen = () => {
					this.connected = true;
					this.connecting = false;
					this.reconnectAttempts = 0;
					this.hasConnectedBefore = true;
					if (isReconnect) {
						this.dispatchLocalEvent('__ws_reconnected__', {});
					}
					resolve();
				};

				ws.onerror = (error) => {
					this.connecting = false;
					reject(error);
				};

				ws.onclose = () => {
					this.connected = false;
					this.connecting = false;
					this.scheduleReconnect();
				};

				ws.onmessage = (event) => {
					this.handleMessage(event.data);
				};
			});
		} catch {
			this.connecting = false;
			this.scheduleReconnect();
			throw new Error('Failed to connect to event WebSocket');
		}
	}

	private handleMessage(data: string): void {
		try {
			const event = JSON.parse(data) as MaestroEvent;
			const callbacks = this.listeners.get(event.event_type);
			if (callbacks) {
				for (const callback of callbacks) {
					try {
						callback(event.payload);
					} catch (e) {
						console.error('Error in event callback:', e);
					}
				}
			}
		} catch (e) {
			console.error('Failed to parse WebSocket message:', e);
		}
	}

	private scheduleReconnect(): void {
		if (this.reconnectTimer) {
			return;
		}

		if (this.listeners.size === 0) {
			return;
		}

		const delay = Math.min(
			1000 * Math.pow(2, this.reconnectAttempts),
			this.maxReconnectDelay
		);
		this.reconnectAttempts++;

		this.reconnectTimer = setTimeout(async () => {
			this.reconnectTimer = null;
			try {
				await this.connect();
			} catch {
				// Will retry via scheduleReconnect called from onclose
			}
		}, delay);
	}

	addListener<T>(eventName: string, callback: EventCallback<T>): void {
		let callbacks = this.listeners.get(eventName);
		if (!callbacks) {
			callbacks = new Set();
			this.listeners.set(eventName, callbacks);
		}
		callbacks.add(callback as EventCallback<unknown>);

		if (!this.connected && !this.connecting) {
			this.connect().catch(() => {
				// Connection will be retried
			});
		}
	}

	removeListener<T>(eventName: string, callback: EventCallback<T>): void {
		const callbacks = this.listeners.get(eventName);
		if (callbacks) {
			callbacks.delete(callback as EventCallback<unknown>);
			if (callbacks.size === 0) {
				this.listeners.delete(eventName);
			}
		}

		if (this.listeners.size === 0) {
			this.disconnect();
		}
	}

	disconnect(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		this.connected = false;
		this.connecting = false;
	}

	private dispatchLocalEvent(eventType: string, payload: unknown): void {
		const callbacks = this.listeners.get(eventType);
		if (callbacks) {
			for (const callback of callbacks) {
				try {
					callback(payload);
				} catch (e) {
					console.error('Error in event callback:', e);
				}
			}
		}
	}
}

let wsManager: WebSocketEventManager | null = null;

function getWebSocketManager(): WebSocketEventManager {
	if (!wsManager) {
		wsManager = new WebSocketEventManager();
	}
	return wsManager;
}

export async function listenEvent<T>(
	eventName: string,
	callback: (payload: T) => void
): Promise<() => void> {
	const mode: TransportMode = getTransportMode();

	switch (mode) {
		case 'tauri': {
			const { listen } = await import('@tauri-apps/api/event');
			const unlisten = await listen<T>(eventName, (event) => {
				callback(event.payload);
			});
			return unlisten;
		}

		case 'http': {
			const manager = getWebSocketManager();
			manager.addListener(eventName, callback);
			return () => {
				manager.removeListener(eventName, callback);
			};
		}

		case 'mock': {
			return () => {
				// No-op for mock mode
			};
		}
	}
}

export function disconnectEventStream(): void {
	if (wsManager) {
		wsManager.disconnect();
		wsManager = null;
	}
}
