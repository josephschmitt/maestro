import { describe, it, expect } from 'vitest';
import { getToolIcon, getToolNames } from './tool-icons.js';

describe('getToolIcon', () => {
	it('returns correct config for known tools', () => {
		const read = getToolIcon('Read');
		expect(read.color).toBe('text-blue-400');
		expect(read.label).toBe('Read');
		expect(read.icon).toBeDefined();

		const bash = getToolIcon('Bash');
		expect(bash.color).toBe('text-purple-400');
		expect(bash.label).toBe('Bash');

		const edit = getToolIcon('Edit');
		expect(edit.color).toBe('text-yellow-400');

		const write = getToolIcon('Write');
		expect(write.color).toBe('text-green-400');

		const grep = getToolIcon('Grep');
		expect(grep.color).toBe('text-orange-400');
	});

	it('returns default config for unknown tools', () => {
		const unknown = getToolIcon('UnknownTool');
		expect(unknown.color).toBe('text-muted-foreground');
		expect(unknown.label).toBe('Tool');
		expect(unknown.icon).toBeDefined();
	});
});

describe('getToolNames', () => {
	it('returns array of known tool names', () => {
		const names = getToolNames();
		expect(names).toContain('Read');
		expect(names).toContain('Write');
		expect(names).toContain('Edit');
		expect(names).toContain('Bash');
		expect(names).toContain('Grep');
		expect(names).toContain('Glob');
		expect(names.length).toBeGreaterThan(0);
	});
});
