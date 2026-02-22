import { writable, derived } from 'svelte/store';
import { getTransportMode, getAuthToken, setAuthToken, clearAuthToken } from '$lib/services/db.js';

interface AuthState {
	requiresToken: boolean;
	hasToken: boolean;
	promptOpen: boolean;
}

function createAuthStore() {
	const state = writable<AuthState>({
		requiresToken: false,
		hasToken: false,
		promptOpen: false
	});

	function initialize() {
		const mode = getTransportMode();
		const token = getAuthToken();

		if (mode === 'http') {
			const hasToken = !!token;
			state.set({
				requiresToken: true,
				hasToken,
				promptOpen: !hasToken
			});
		} else {
			state.set({
				requiresToken: false,
				hasToken: false,
				promptOpen: false
			});
		}
	}

	function saveToken(token: string) {
		setAuthToken(token);
		state.update((s) => ({
			...s,
			hasToken: true,
			promptOpen: false
		}));
		window.location.reload();
	}

	function clearToken() {
		clearAuthToken();
		state.update((s) => ({
			...s,
			hasToken: false,
			promptOpen: true
		}));
	}

	function openPrompt() {
		state.update((s) => ({
			...s,
			promptOpen: true
		}));
	}

	return {
		subscribe: state.subscribe,
		initialize,
		saveToken,
		clearToken,
		openPrompt
	};
}

export const authStore = createAuthStore();

export const authPromptOpen = derived(authStore, ($auth) => $auth.promptOpen);
export const requiresAuth = derived(authStore, ($auth) => $auth.requiresToken);
export const hasAuthToken = derived(authStore, ($auth) => $auth.hasToken);
