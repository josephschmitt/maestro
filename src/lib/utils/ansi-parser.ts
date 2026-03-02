// eslint-disable-next-line no-control-regex
const ANSI_REGEX = /\x1b\[([0-9;]*)m/g;

const COLORS_16: Record<number, string> = {
	30: '#1a1a1a',
	31: '#e06c75',
	32: '#98c379',
	33: '#e5c07b',
	34: '#61afef',
	35: '#c678dd',
	36: '#56b6c2',
	37: '#abb2bf',
	90: '#5c6370',
	91: '#e06c75',
	92: '#98c379',
	93: '#e5c07b',
	94: '#61afef',
	95: '#c678dd',
	96: '#56b6c2',
	97: '#ffffff'
};

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;');
}

export function parseAnsi(text: string): string {
	let bold = false;
	let color: string | null = null;
	let result = '';
	let lastIndex = 0;

	let match: RegExpExecArray | null;
	ANSI_REGEX.lastIndex = 0;

	while ((match = ANSI_REGEX.exec(text)) !== null) {
		const before = text.slice(lastIndex, match.index);
		if (before) {
			result += wrapSpan(escapeHtml(before), bold, color);
		}
		lastIndex = match.index + match[0].length;

		const codes = match[1].split(';').filter(Boolean).map(Number);
		if (codes.length === 0) codes.push(0);

		for (const code of codes) {
			if (code === 0) {
				bold = false;
				color = null;
			} else if (code === 1) {
				bold = true;
			} else if (code === 22) {
				bold = false;
			} else if (code >= 30 && code <= 37) {
				color = COLORS_16[code] ?? null;
			} else if (code >= 90 && code <= 97) {
				color = COLORS_16[code] ?? null;
			} else if (code === 39) {
				color = null;
			}
		}
	}

	const remaining = text.slice(lastIndex);
	if (remaining) {
		result += wrapSpan(escapeHtml(remaining), bold, color);
	}

	return result;
}

function wrapSpan(html: string, bold: boolean, color: string | null): string {
	if (!bold && !color) return html;
	const styles: string[] = [];
	if (bold) styles.push('font-weight:700');
	if (color) styles.push(`color:${color}`);
	return `<span style="${styles.join(';')}">${html}</span>`;
}
