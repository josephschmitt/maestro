import { describe, it, expect } from 'vitest';
import { parseAnsi } from './ansi-parser.js';

describe('parseAnsi', () => {
	it('returns plain text unchanged (escaped)', () => {
		expect(parseAnsi('hello world')).toBe('hello world');
	});

	it('escapes HTML in input', () => {
		expect(parseAnsi('<div>test</div>')).toBe('&lt;div&gt;test&lt;/div&gt;');
	});

	it('parses bold (SGR 1)', () => {
		const result = parseAnsi('\x1b[1mbold text\x1b[0m');
		expect(result).toContain('font-weight:700');
		expect(result).toContain('bold text');
	});

	it('parses foreground colors (30-37)', () => {
		const result = parseAnsi('\x1b[31mred text\x1b[0m');
		expect(result).toContain('color:#e06c75');
		expect(result).toContain('red text');
	});

	it('parses bright colors (90-97)', () => {
		const result = parseAnsi('\x1b[92mgreen text\x1b[0m');
		expect(result).toContain('color:#98c379');
		expect(result).toContain('green text');
	});

	it('handles combined bold and color', () => {
		const result = parseAnsi('\x1b[1;33myellow bold\x1b[0m');
		expect(result).toContain('font-weight:700');
		expect(result).toContain('color:#e5c07b');
		expect(result).toContain('yellow bold');
	});

	it('resets with SGR 0', () => {
		const result = parseAnsi('\x1b[31mred\x1b[0m normal');
		expect(result).toContain('red');
		expect(result).toContain(' normal');
		// "normal" should not be in a styled span
		expect(result).toMatch(/<\/span> normal$/);
	});

	it('handles multiple color changes', () => {
		const result = parseAnsi('\x1b[31mred\x1b[32mgreen\x1b[0m');
		expect(result).toContain('#e06c75');
		expect(result).toContain('#98c379');
	});

	it('handles text with no ANSI codes', () => {
		const input = 'just plain text with special chars: & < >';
		const result = parseAnsi(input);
		expect(result).toBe('just plain text with special chars: &amp; &lt; &gt;');
	});

	it('handles empty string', () => {
		expect(parseAnsi('')).toBe('');
	});

	it('handles reset color (SGR 39)', () => {
		const result = parseAnsi('\x1b[31mred\x1b[39mdefault');
		expect(result).toContain('#e06c75');
		// After reset, "default" should not have a color style
		expect(result).toMatch(/default$/);
	});

	it('handles reset bold (SGR 22)', () => {
		const result = parseAnsi('\x1b[1mbold\x1b[22mnormal');
		const parts = result.split('</span>');
		expect(parts[0]).toContain('font-weight:700');
		// "normal" should not be bold
		expect(result).toMatch(/normal$/);
	});
});
