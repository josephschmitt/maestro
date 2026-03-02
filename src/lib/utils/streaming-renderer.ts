import { renderMarkdown } from './markdown.js';

const THROTTLE_MS = 150;

export interface StreamingRenderer {
	update(content: string): void;
	flush(): void;
	destroy(): void;
	readonly html: string;
	subscribe(callback: (html: string) => void): () => void;
}

export function createStreamingRenderer(): StreamingRenderer {
	let currentContent = '';
	let lastRenderedContent = '';
	let renderedHtml = '';
	let lastRenderTime = 0;
	let rafId: number | null = null;
	let timeoutId: ReturnType<typeof setTimeout> | null = null;
	const listeners: Set<(html: string) => void> = new Set();

	function notify() {
		for (const cb of listeners) {
			cb(renderedHtml);
		}
	}

	function doRender() {
		if (currentContent === lastRenderedContent) return;
		lastRenderedContent = currentContent;
		renderedHtml = renderMarkdown(currentContent);
		lastRenderTime = performance.now();
		notify();
	}

	function cancelPending() {
		if (rafId !== null) {
			cancelAnimationFrame(rafId);
			rafId = null;
		}
		if (timeoutId !== null) {
			clearTimeout(timeoutId);
			timeoutId = null;
		}
	}

	function scheduleRender() {
		if (rafId !== null) return;

		rafId = requestAnimationFrame(() => {
			rafId = null;
			const elapsed = performance.now() - lastRenderTime;
			if (elapsed >= THROTTLE_MS) {
				doRender();
			} else {
				const remaining = THROTTLE_MS - elapsed;
				timeoutId = setTimeout(() => {
					timeoutId = null;
					rafId = requestAnimationFrame(() => {
						rafId = null;
						doRender();
					});
				}, remaining);
			}
		});
	}

	return {
		update(content: string) {
			currentContent = content;
			scheduleRender();
		},

		flush() {
			cancelPending();
			doRender();
		},

		destroy() {
			cancelPending();
			listeners.clear();
		},

		get html() {
			return renderedHtml;
		},

		subscribe(callback: (html: string) => void) {
			listeners.add(callback);
			if (renderedHtml) callback(renderedHtml);
			return () => listeners.delete(callback);
		}
	};
}
