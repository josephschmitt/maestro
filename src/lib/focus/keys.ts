export const Keys = {
	Escape: 'Escape',
	Tab: 'Tab',
	Enter: 'Enter',
	Space: ' ',
	ArrowUp: 'ArrowUp',
	ArrowDown: 'ArrowDown',
	ArrowLeft: 'ArrowLeft',
	ArrowRight: 'ArrowRight'
} as const;

export type Key = (typeof Keys)[keyof typeof Keys];

export function isKey(event: KeyboardEvent, key: Key): boolean {
	return event.key === key;
}

export function isEscape(event: KeyboardEvent): boolean {
	return isKey(event, Keys.Escape);
}

export function isActivation(event: KeyboardEvent): boolean {
	return isKey(event, Keys.Enter) || isKey(event, Keys.Space);
}
