import { writable, derived, get } from 'svelte/store';
import type { FocusRegionId, FocusElement, FocusStackEntry, FocusState } from './types.js';

const initialState: FocusState = {
	activeRegion: null,
	stack: [],
	elementsByRegion: new Map()
};

export const focusState = writable<FocusState>(initialState);

export const activeRegion = derived(focusState, ($state) => $state.activeRegion);
export const focusStack = derived(focusState, ($state) => $state.stack);

export function registerElement(element: FocusElement): void {
	focusState.update((state) => {
		const regionElements = state.elementsByRegion.get(element.region) ?? [];
		if (regionElements.some((e) => e.id === element.id)) {
			return state;
		}
		const updated = new Map(state.elementsByRegion);
		updated.set(element.region, [...regionElements, element]);
		return { ...state, elementsByRegion: updated };
	});
}

export function unregisterElement(elementId: string, region: FocusRegionId): void {
	focusState.update((state) => {
		const regionElements = state.elementsByRegion.get(region);
		if (!regionElements) return state;
		const filtered = regionElements.filter((e) => e.id !== elementId);
		const updated = new Map(state.elementsByRegion);
		if (filtered.length === 0) {
			updated.delete(region);
		} else {
			updated.set(region, filtered);
		}
		return { ...state, elementsByRegion: updated };
	});
}

export function setActiveRegion(region: FocusRegionId): void {
	focusState.update((state) => ({ ...state, activeRegion: region }));
}

export function pushRegion(region: FocusRegionId): void {
	focusState.update((state) => {
		const previouslyFocusedElement =
			document.activeElement instanceof HTMLElement ? document.activeElement : null;
		const entry: FocusStackEntry = {
			region: state.activeRegion ?? region,
			previouslyFocusedElement
		};
		return {
			...state,
			activeRegion: region,
			stack: [...state.stack, entry]
		};
	});
}

export function popRegion(): boolean {
	const state = get(focusState);
	if (state.stack.length === 0) return false;

	const entry = state.stack[state.stack.length - 1];
	focusState.update((s) => ({
		...s,
		activeRegion: entry.region,
		stack: s.stack.slice(0, -1)
	}));

	if (entry.previouslyFocusedElement && entry.previouslyFocusedElement.isConnected) {
		entry.previouslyFocusedElement.focus();
	}

	return true;
}

export function getRegionElements(region: FocusRegionId): FocusElement[] {
	return get(focusState).elementsByRegion.get(region) ?? [];
}
