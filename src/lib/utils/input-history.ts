const MAX_HISTORY = 50;

export interface InputHistory {
	add: (entry: string) => void;
	prev: (currentDraft: string) => string | null;
	next: () => string | null;
	current: () => string;
	reset: () => void;
}

export function createInputHistory(): InputHistory {
	let entries: string[] = [];
	let index = -1;
	let draft = '';

	function add(entry: string) {
		const trimmed = entry.trim();
		if (!trimmed) return;
		if (entries[0] === trimmed) {
			index = -1;
			draft = '';
			return;
		}
		entries.unshift(trimmed);
		if (entries.length > MAX_HISTORY) {
			entries = entries.slice(0, MAX_HISTORY);
		}
		index = -1;
		draft = '';
	}

	function prev(currentDraft: string): string | null {
		if (entries.length === 0) return null;
		if (index === -1) {
			draft = currentDraft;
		}
		if (index < entries.length - 1) {
			index++;
			return entries[index];
		}
		return null;
	}

	function next(): string | null {
		if (index <= -1) return null;
		index--;
		if (index === -1) {
			return draft;
		}
		return entries[index];
	}

	function current(): string {
		if (index === -1) return draft;
		return entries[index];
	}

	function reset() {
		index = -1;
		draft = '';
	}

	return { add, prev, next, current, reset };
}
