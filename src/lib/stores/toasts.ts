import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
	id: string;
	type: ToastType;
	title: string;
	message?: string;
	duration: number;
}

const DEFAULT_DURATIONS: Record<ToastType, number> = {
	success: 5000,
	info: 5000,
	warning: 8000,
	error: 0, // Sticky - no auto-dismiss
};

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	let nextId = 1;

	function addToast(
		type: ToastType,
		title: string,
		message?: string,
		duration?: number
	): string {
		const id = `toast-${nextId++}`;
		const toast: Toast = {
			id,
			type,
			title,
			message,
			duration: duration ?? DEFAULT_DURATIONS[type],
		};

		update((toasts) => [toast, ...toasts]);

		if (toast.duration > 0) {
			setTimeout(() => {
				dismiss(id);
			}, toast.duration);
		}

		return id;
	}

	function dismiss(id: string) {
		update((toasts) => toasts.filter((t) => t.id !== id));
	}

	function clear() {
		update(() => []);
	}

	return {
		subscribe,
		addToast,
		dismiss,
		clear,
		success: (title: string, message?: string) => addToast('success', title, message),
		error: (title: string, message?: string) => addToast('error', title, message),
		warning: (title: string, message?: string) => addToast('warning', title, message),
		info: (title: string, message?: string) => addToast('info', title, message),
	};
}

export const toasts = createToastStore();
