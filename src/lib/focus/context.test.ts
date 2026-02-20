// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
	focusState,
	registerElement,
	unregisterElement,
	setActiveRegion,
	pushRegion,
	popRegion,
	getRegionElements
} from './context.js';
import type { FocusElement } from './types.js';

function resetStore() {
	focusState.set({
		activeRegion: null,
		stack: [],
		elementsByRegion: new Map()
	});
}

function makeElement(id: string): HTMLElement {
	const el = document.createElement('div');
	el.id = id;
	document.body.appendChild(el);
	return el;
}

describe('focus context store', () => {
	beforeEach(() => {
		resetStore();
		document.body.innerHTML = '';
	});

	describe('registerElement / unregisterElement', () => {
		it('registers an element in its region', () => {
			const el = makeElement('card-1');
			registerElement({ id: 'card-1', region: 'board', element: el });

			const state = get(focusState);
			expect(state.elementsByRegion.get('board')).toHaveLength(1);
			expect(state.elementsByRegion.get('board')![0].id).toBe('card-1');
		});

		it('does not duplicate elements with the same id', () => {
			const el = makeElement('card-1');
			registerElement({ id: 'card-1', region: 'board', element: el });
			registerElement({ id: 'card-1', region: 'board', element: el });

			const state = get(focusState);
			expect(state.elementsByRegion.get('board')).toHaveLength(1);
		});

		it('unregisters an element', () => {
			const el = makeElement('card-1');
			registerElement({ id: 'card-1', region: 'board', element: el });
			unregisterElement('card-1', 'board');

			const state = get(focusState);
			expect(state.elementsByRegion.has('board')).toBe(false);
		});

		it('removes region key when last element is unregistered', () => {
			const el = makeElement('card-1');
			registerElement({ id: 'card-1', region: 'board', element: el });
			unregisterElement('card-1', 'board');

			const state = get(focusState);
			expect(state.elementsByRegion.has('board')).toBe(false);
		});
	});

	describe('setActiveRegion', () => {
		it('sets the active region', () => {
			setActiveRegion('board');
			expect(get(focusState).activeRegion).toBe('board');
		});
	});

	describe('pushRegion / popRegion', () => {
		it('pushes a region onto the stack', () => {
			setActiveRegion('board');
			pushRegion('card-detail');

			const state = get(focusState);
			expect(state.activeRegion).toBe('card-detail');
			expect(state.stack).toHaveLength(1);
			expect(state.stack[0].region).toBe('board');
		});

		it('pops a region and restores the previous one', () => {
			setActiveRegion('board');
			pushRegion('card-detail');

			const popped = popRegion();
			expect(popped).toBe(true);

			const state = get(focusState);
			expect(state.activeRegion).toBe('board');
			expect(state.stack).toHaveLength(0);
		});

		it('returns false when popping an empty stack', () => {
			expect(popRegion()).toBe(false);
		});

		it('restores focus to previously focused element on pop', () => {
			const el = makeElement('btn');
			el.setAttribute('tabindex', '0');
			el.focus();

			setActiveRegion('board');
			pushRegion('card-detail');
			popRegion();

			expect(document.activeElement).toBe(el);
		});

		it('handles nested pushes correctly', () => {
			setActiveRegion('board');
			pushRegion('card-detail');
			pushRegion('dialog');

			const state = get(focusState);
			expect(state.activeRegion).toBe('dialog');
			expect(state.stack).toHaveLength(2);

			popRegion();
			expect(get(focusState).activeRegion).toBe('card-detail');

			popRegion();
			expect(get(focusState).activeRegion).toBe('board');
		});
	});

	describe('getRegionElements', () => {
		it('returns elements for a region', () => {
			const el = makeElement('card-1');
			registerElement({ id: 'card-1', region: 'board', element: el });

			const elements = getRegionElements('board');
			expect(elements).toHaveLength(1);
		});

		it('returns empty array for unknown region', () => {
			expect(getRegionElements('sidebar')).toEqual([]);
		});
	});
});
