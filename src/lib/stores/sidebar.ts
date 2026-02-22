import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const STORAGE_KEY = 'maestro-sidebar-width';
const DEFAULT_WIDTH = 324;
const MIN_WIDTH = 200;
const MAX_WIDTH = 500;

function getInitialWidth(): number {
	if (!browser) return DEFAULT_WIDTH;
	const stored = localStorage.getItem(STORAGE_KEY);
	if (stored) {
		const parsed = parseInt(stored, 10);
		if (!isNaN(parsed) && parsed >= MIN_WIDTH && parsed <= MAX_WIDTH) {
			return parsed;
		}
	}
	return DEFAULT_WIDTH;
}

function createSidebarWidthStore() {
	const { subscribe, set, update } = writable(getInitialWidth());

	return {
		subscribe,
		setWidth: (width: number) => {
			const clamped = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, width));
			set(clamped);
			if (browser) {
				localStorage.setItem(STORAGE_KEY, String(clamped));
			}
		},
		reset: () => {
			set(DEFAULT_WIDTH);
			if (browser) {
				localStorage.setItem(STORAGE_KEY, String(DEFAULT_WIDTH));
			}
		}
	};
}

export const sidebarWidth = createSidebarWidthStore();
export const SIDEBAR_MIN_WIDTH = MIN_WIDTH;
export const SIDEBAR_MAX_WIDTH = MAX_WIDTH;
