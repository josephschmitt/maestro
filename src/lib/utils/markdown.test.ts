// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { renderMarkdown } from './markdown.js';

describe('renderMarkdown', () => {
	it('renders plain text as a paragraph', () => {
		const result = renderMarkdown('Hello world');
		expect(result).toContain('<p>Hello world</p>');
	});

	it('renders inline code', () => {
		const result = renderMarkdown('Use `console.log()` here');
		expect(result).toContain('<code>console.log()</code>');
	});

	it('renders a fenced code block with language hint', () => {
		const result = renderMarkdown('```typescript\nconst x = 1;\n```');
		expect(result).toContain('class="code-block-wrapper"');
		expect(result).toContain('data-lang="TypeScript"');
		expect(result).toContain('class="hljs"');
	});

	it('renders a fenced code block without language hint', () => {
		const result = renderMarkdown('```\nhello world\n```');
		expect(result).toContain('class="code-block-wrapper"');
		expect(result).toContain('data-lang="Text"');
	});

	it('applies syntax highlighting to known languages', () => {
		const result = renderMarkdown('```javascript\nfunction foo() { return 42; }\n```');
		expect(result).toContain('hljs-keyword');
	});

	it('handles nested markdown (bold, italic, lists)', () => {
		const result = renderMarkdown('**bold** and *italic*\n\n- item 1\n- item 2');
		expect(result).toContain('<strong>bold</strong>');
		expect(result).toContain('<em>italic</em>');
		expect(result).toContain('<li>');
	});

	it('sanitizes HTML to prevent XSS', () => {
		const result = renderMarkdown('<script>alert("xss")</script>');
		expect(result).not.toContain('<script>');
	});

	it('preserves data attributes for code blocks', () => {
		const result = renderMarkdown('```python\nprint("hi")\n```');
		expect(result).toContain('data-raw-code');
		expect(result).toContain('data-lang="Python"');
	});

	it('handles language aliases', () => {
		const result = renderMarkdown('```ts\nconst x: number = 1;\n```');
		expect(result).toContain('data-lang="TypeScript"');
	});

	it('renders multiple code blocks', () => {
		const md = '```js\nconst a = 1;\n```\n\nSome text\n\n```rust\nfn main() {}\n```';
		const result = renderMarkdown(md);
		const wrappers = result.match(/code-block-wrapper/g);
		expect(wrappers?.length).toBe(2);
	});
});
