import FileTextIcon from '@lucide/svelte/icons/file-text';
import PencilIcon from '@lucide/svelte/icons/pencil';
import TerminalIcon from '@lucide/svelte/icons/terminal';
import SearchIcon from '@lucide/svelte/icons/search';
import FilePenIcon from '@lucide/svelte/icons/file-pen';
import FolderSearchIcon from '@lucide/svelte/icons/folder-search';
import GlobeIcon from '@lucide/svelte/icons/globe';
import WrenchIcon from '@lucide/svelte/icons/wrench';
import type { Component } from 'svelte';

export interface ToolIconConfig {
	icon: Component;
	color: string;
	label: string;
}

const TOOL_ICON_MAP: Record<string, ToolIconConfig> = {
	Read: { icon: FileTextIcon, color: 'text-blue-400', label: 'Read' },
	Write: { icon: FilePenIcon, color: 'text-green-400', label: 'Write' },
	Edit: { icon: PencilIcon, color: 'text-yellow-400', label: 'Edit' },
	Bash: { icon: TerminalIcon, color: 'text-purple-400', label: 'Bash' },
	Grep: { icon: SearchIcon, color: 'text-orange-400', label: 'Grep' },
	Glob: { icon: FolderSearchIcon, color: 'text-teal-400', label: 'Glob' },
	WebFetch: { icon: GlobeIcon, color: 'text-cyan-400', label: 'WebFetch' },
	WebSearch: { icon: GlobeIcon, color: 'text-cyan-400', label: 'WebSearch' },
	Task: { icon: WrenchIcon, color: 'text-pink-400', label: 'Task' }
};

const DEFAULT_TOOL_ICON: ToolIconConfig = {
	icon: WrenchIcon,
	color: 'text-muted-foreground',
	label: 'Tool'
};

export function getToolIcon(toolName: string): ToolIconConfig {
	return TOOL_ICON_MAP[toolName] ?? DEFAULT_TOOL_ICON;
}

export function getToolNames(): string[] {
	return Object.keys(TOOL_ICON_MAP);
}
