export interface ToolStartEvent {
	type: 'tool_start';
	id: string;
	tool_name: string;
	input_summary: string;
}

export interface ToolEndEvent {
	type: 'tool_end';
	id: string;
	output_preview?: string;
	output_full?: string;
	error?: string;
}

export type ToolEvent = ToolStartEvent | ToolEndEvent;

const TOOL_START_PREFIX = '{"maestro_tool":"start"';
const TOOL_END_PREFIX = '{"maestro_tool":"end"';

export function parseToolEvent(line: string): ToolEvent | null {
	const trimmed = line.trim();
	if (!trimmed.startsWith('{')) return null;

	if (!trimmed.includes('"maestro_tool"')) return null;

	try {
		const parsed = JSON.parse(trimmed);
		if (parsed.maestro_tool === 'start' && parsed.id && parsed.tool_name) {
			return {
				type: 'tool_start',
				id: parsed.id,
				tool_name: parsed.tool_name,
				input_summary: parsed.input_summary ?? ''
			};
		}
		if (parsed.maestro_tool === 'end' && parsed.id) {
			return {
				type: 'tool_end',
				id: parsed.id,
				output_preview: parsed.output_preview,
				output_full: parsed.output_full,
				error: parsed.error
			};
		}
	} catch {
		return null;
	}

	return null;
}

export function isToolEventLine(line: string): boolean {
	const trimmed = line.trim();
	return trimmed.includes('"maestro_tool"') && trimmed.startsWith('{');
}
