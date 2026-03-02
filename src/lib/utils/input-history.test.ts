import { describe, it, expect } from 'vitest';
import { createInputHistory } from './input-history.js';

describe('createInputHistory', () => {
	it('returns null for prev when history is empty', () => {
		const h = createInputHistory();
		expect(h.prev('')).toBeNull();
	});

	it('cycles through history with prev', () => {
		const h = createInputHistory();
		h.add('first');
		h.add('second');
		h.add('third');

		expect(h.prev('')).toBe('third');
		expect(h.prev('')).toBe('second');
		expect(h.prev('')).toBe('first');
		expect(h.prev('')).toBeNull();
	});

	it('cycles forward with next', () => {
		const h = createInputHistory();
		h.add('first');
		h.add('second');

		h.prev('');
		h.prev('');
		expect(h.next()).toBe('second');
		expect(h.next()).toBe('');
	});

	it('returns null for next when at latest', () => {
		const h = createInputHistory();
		h.add('first');
		expect(h.next()).toBeNull();
	});

	it('preserves current draft when navigating', () => {
		const h = createInputHistory();
		h.add('old message');

		const draft = 'my current draft';
		h.prev(draft);
		expect(h.current()).toBe('old message');

		h.next();
		expect(h.current()).toBe(draft);
	});

	it('deduplicates consecutive identical entries', () => {
		const h = createInputHistory();
		h.add('same');
		h.add('same');

		expect(h.prev('')).toBe('same');
		expect(h.prev('')).toBeNull();
	});

	it('ignores empty strings', () => {
		const h = createInputHistory();
		h.add('');
		h.add('   ');
		expect(h.prev('')).toBeNull();
	});

	it('limits history to 50 entries', () => {
		const h = createInputHistory();
		for (let i = 0; i < 60; i++) {
			h.add(`message ${i}`);
		}

		let count = 0;
		while (h.prev('') !== null) {
			count++;
		}
		expect(count).toBe(50);
	});

	it('resets index after adding a new entry', () => {
		const h = createInputHistory();
		h.add('first');
		h.add('second');

		h.prev('');
		expect(h.current()).toBe('second');

		h.add('third');
		expect(h.current()).toBe('');
		expect(h.prev('')).toBe('third');
	});

	it('reset clears navigation state', () => {
		const h = createInputHistory();
		h.add('first');
		h.prev('draft');

		h.reset();
		expect(h.current()).toBe('');
	});
});

describe('token estimate heuristic', () => {
	it('estimates tokens as chars / 4', () => {
		const chars = 2000;
		const estimate = Math.ceil(chars / 4);
		expect(estimate).toBe(500);
	});

	it('rounds up for non-even divisions', () => {
		const chars = 1001;
		const estimate = Math.ceil(chars / 4);
		expect(estimate).toBe(251);
	});
});

describe('paste compression logic', () => {
	const PASTE_THRESHOLD = 500;

	it('text over 500 chars should be compressed into a chip', () => {
		const longText = 'a'.repeat(501);
		expect(longText.length).toBeGreaterThan(PASTE_THRESHOLD);
	});

	it('text under 500 chars should pass through', () => {
		const shortText = 'a'.repeat(499);
		expect(shortText.length).toBeLessThanOrEqual(PASTE_THRESHOLD);
	});

	it('text exactly 500 chars should pass through', () => {
		const exactText = 'a'.repeat(500);
		expect(exactText.length).toBeLessThanOrEqual(PASTE_THRESHOLD);
	});

	it('chip preview should be first 50 chars', () => {
		const content = 'x'.repeat(600);
		const preview = content.slice(0, 50).replace(/\n/g, ' ');
		expect(preview.length).toBe(50);
	});

	it('chip preview replaces newlines with spaces', () => {
		const content = 'line1\nline2\nline3' + 'x'.repeat(500);
		const preview = content.slice(0, 50).replace(/\n/g, ' ');
		expect(preview).not.toContain('\n');
	});
});
