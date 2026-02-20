// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { Keys, isKey, isEscape, isActivation } from './keys.js';

function makeKeyEvent(key: string): KeyboardEvent {
	return new KeyboardEvent('keydown', { key });
}

describe('key helpers', () => {
	describe('isKey', () => {
		it('returns true for matching key', () => {
			expect(isKey(makeKeyEvent('Escape'), Keys.Escape)).toBe(true);
		});

		it('returns false for non-matching key', () => {
			expect(isKey(makeKeyEvent('Enter'), Keys.Escape)).toBe(false);
		});
	});

	describe('isEscape', () => {
		it('returns true for Escape', () => {
			expect(isEscape(makeKeyEvent('Escape'))).toBe(true);
		});

		it('returns false for other keys', () => {
			expect(isEscape(makeKeyEvent('Enter'))).toBe(false);
		});
	});

	describe('isActivation', () => {
		it('returns true for Enter', () => {
			expect(isActivation(makeKeyEvent('Enter'))).toBe(true);
		});

		it('returns true for Space', () => {
			expect(isActivation(makeKeyEvent(' '))).toBe(true);
		});

		it('returns false for other keys', () => {
			expect(isActivation(makeKeyEvent('Tab'))).toBe(false);
		});
	});
});
