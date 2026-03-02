import { describe, it, expect } from 'vitest';
import { buildTimeline } from './agent.js';
import type { AgentOutputLine, ToolInvocation } from '$lib/types/index.js';

function line(text: string, stream: 'stdout' | 'stderr' = 'stdout'): AgentOutputLine {
	return { stream, line: text };
}

function toolStart(id: string, name: string): AgentOutputLine {
	return line(JSON.stringify({ maestro_tool: 'start', id, tool_name: name, input_summary: 'test' }));
}

function toolEnd(id: string): AgentOutputLine {
	return line(JSON.stringify({ maestro_tool: 'end', id, output_preview: 'output' }));
}

function makeInvocation(id: string, name: string, status: 'running' | 'completed' = 'completed'): ToolInvocation {
	return {
		id,
		tool_name: name,
		status,
		started_at: new Date().toISOString(),
		input_summary: 'test',
		output_preview: 'output'
	};
}

describe('buildTimeline', () => {
	it('returns empty array for no input', () => {
		const result = buildTimeline([], new Map());
		expect(result).toEqual([]);
	});

	it('groups plain text lines into a single text block', () => {
		const lines = [line('hello'), line('world')];
		const result = buildTimeline(lines, new Map());
		expect(result).toHaveLength(1);
		expect(result[0].type).toBe('text');
		if (result[0].type === 'text') {
			expect(result[0].lines).toHaveLength(2);
		}
	});

	it('interleaves text and tool entries', () => {
		const tools = new Map<string, ToolInvocation>();
		tools.set('t1', makeInvocation('t1', 'Read'));

		const lines = [
			line('before'),
			toolStart('t1', 'Read'),
			toolEnd('t1'),
			line('after')
		];

		const result = buildTimeline(lines, tools);
		expect(result.length).toBeGreaterThanOrEqual(3);
		expect(result[0].type).toBe('text');
		expect(result.some(e => e.type === 'tool')).toBe(true);
		expect(result[result.length - 1].type).toBe('text');
	});

	it('handles consecutive tools without text between them', () => {
		const tools = new Map<string, ToolInvocation>();
		tools.set('t1', makeInvocation('t1', 'Read'));
		tools.set('t2', makeInvocation('t2', 'Edit'));

		const lines = [
			toolStart('t1', 'Read'),
			toolEnd('t1'),
			toolStart('t2', 'Edit'),
			toolEnd('t2')
		];

		const result = buildTimeline(lines, tools);
		const toolEntries = result.filter(e => e.type === 'tool');
		expect(toolEntries).toHaveLength(2);
	});

	it('handles stderr lines in text blocks', () => {
		const lines = [
			line('normal'),
			line('error!', 'stderr'),
			line('back to normal')
		];
		const result = buildTimeline(lines, new Map());
		expect(result).toHaveLength(1);
		if (result[0].type === 'text') {
			expect(result[0].lines[1].stream).toBe('stderr');
		}
	});
});
