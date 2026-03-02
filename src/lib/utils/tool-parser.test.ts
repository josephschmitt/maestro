import { describe, it, expect } from 'vitest';
import { parseToolEvent, isToolEventLine } from './tool-parser.js';

describe('parseToolEvent', () => {
	it('parses a tool start event', () => {
		const line = '{"maestro_tool":"start","id":"t1","tool_name":"Read","input_summary":"src/main.ts"}';
		const event = parseToolEvent(line);
		expect(event).toEqual({
			type: 'tool_start',
			id: 't1',
			tool_name: 'Read',
			input_summary: 'src/main.ts'
		});
	});

	it('parses a tool end event', () => {
		const line = '{"maestro_tool":"end","id":"t1","output_preview":"content here"}';
		const event = parseToolEvent(line);
		expect(event).toEqual({
			type: 'tool_end',
			id: 't1',
			output_preview: 'content here',
			output_full: undefined,
			error: undefined
		});
	});

	it('parses a tool end event with error', () => {
		const line = '{"maestro_tool":"end","id":"t2","error":"File not found"}';
		const event = parseToolEvent(line);
		expect(event).not.toBeNull();
		expect(event!.type).toBe('tool_end');
		if (event!.type === 'tool_end') {
			expect(event!.error).toBe('File not found');
		}
	});

	it('returns null for non-tool lines', () => {
		expect(parseToolEvent('hello world')).toBeNull();
		expect(parseToolEvent('')).toBeNull();
		expect(parseToolEvent('{"other":"json"}')).toBeNull();
	});

	it('returns null for malformed JSON', () => {
		expect(parseToolEvent('{maestro_tool:start}')).toBeNull();
	});

	it('handles whitespace around lines', () => {
		const line = '  {"maestro_tool":"start","id":"t1","tool_name":"Bash","input_summary":"ls"}  ';
		const event = parseToolEvent(line);
		expect(event).not.toBeNull();
		expect(event!.type).toBe('tool_start');
	});

	it('handles missing input_summary gracefully', () => {
		const line = '{"maestro_tool":"start","id":"t1","tool_name":"Read"}';
		const event = parseToolEvent(line);
		expect(event).not.toBeNull();
		if (event!.type === 'tool_start') {
			expect(event!.input_summary).toBe('');
		}
	});
});

describe('isToolEventLine', () => {
	it('returns true for tool event lines', () => {
		expect(isToolEventLine('{"maestro_tool":"start","id":"t1","tool_name":"Read"}')).toBe(true);
		expect(isToolEventLine('{"maestro_tool":"end","id":"t1"}')).toBe(true);
	});

	it('returns false for non-tool lines', () => {
		expect(isToolEventLine('regular text')).toBe(false);
		expect(isToolEventLine('{"other":"json"}')).toBe(false);
		expect(isToolEventLine('')).toBe(false);
	});
});
