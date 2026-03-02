import { Marked } from 'marked';
import hljs from 'highlight.js/lib/core';
import DOMPurify from 'dompurify';

import typescript from 'highlight.js/lib/languages/typescript';
import javascript from 'highlight.js/lib/languages/javascript';
import python from 'highlight.js/lib/languages/python';
import rust from 'highlight.js/lib/languages/rust';
import go from 'highlight.js/lib/languages/go';
import json from 'highlight.js/lib/languages/json';
import yaml from 'highlight.js/lib/languages/yaml';
import sql from 'highlight.js/lib/languages/sql';
import bash from 'highlight.js/lib/languages/bash';
import xml from 'highlight.js/lib/languages/xml';
import css from 'highlight.js/lib/languages/css';
import markdown from 'highlight.js/lib/languages/markdown';
import diff from 'highlight.js/lib/languages/diff';
import ini from 'highlight.js/lib/languages/ini';
import c from 'highlight.js/lib/languages/c';

hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('python', python);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('go', go);
hljs.registerLanguage('json', json);
hljs.registerLanguage('yaml', yaml);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('bash', bash);
hljs.registerLanguage('xml', xml);
hljs.registerLanguage('html', xml);
hljs.registerLanguage('css', css);
hljs.registerLanguage('markdown', markdown);
hljs.registerLanguage('diff', diff);
hljs.registerLanguage('toml', ini);
hljs.registerLanguage('c', c);

hljs.registerAliases(['ts', 'tsx', 'jsx'], { languageName: 'typescript' });
hljs.registerAliases(['js'], { languageName: 'javascript' });
hljs.registerAliases(['py'], { languageName: 'python' });
hljs.registerAliases(['rs'], { languageName: 'rust' });
hljs.registerAliases(['sh', 'shell', 'zsh', 'fish'], { languageName: 'bash' });
hljs.registerAliases(['yml'], { languageName: 'yaml' });
hljs.registerAliases(['svelte', 'vue'], { languageName: 'xml' });
hljs.registerAliases(['md'], { languageName: 'markdown' });

const LANGUAGE_LABELS: Record<string, string> = {
	typescript: 'TypeScript',
	ts: 'TypeScript',
	tsx: 'TypeScript',
	javascript: 'JavaScript',
	js: 'JavaScript',
	jsx: 'JavaScript',
	python: 'Python',
	py: 'Python',
	rust: 'Rust',
	rs: 'Rust',
	go: 'Go',
	json: 'JSON',
	yaml: 'YAML',
	yml: 'YAML',
	sql: 'SQL',
	bash: 'Bash',
	sh: 'Bash',
	shell: 'Bash',
	zsh: 'Bash',
	fish: 'Bash',
	html: 'HTML',
	xml: 'XML',
	css: 'CSS',
	svelte: 'Svelte',
	vue: 'Vue',
	markdown: 'Markdown',
	md: 'Markdown',
	diff: 'Diff',
	toml: 'TOML',
	c: 'C'
};

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

const marked = new Marked({
	renderer: {
		code({ text, lang }) {
			const language = lang?.trim().toLowerCase() ?? '';
			const label = LANGUAGE_LABELS[language] ?? (language.toUpperCase() || 'Text');
			let highlighted: string;

			try {
				if (language && hljs.getLanguage(language)) {
					highlighted = hljs.highlight(text, { language }).value;
				} else {
					const result = hljs.highlightAuto(text);
					highlighted = result.value;
				}
			} catch {
				highlighted = escapeHtml(text);
			}

			const escapedRaw = escapeHtml(text);
			return `<div class="code-block-wrapper" data-lang="${escapeHtml(label)}" data-raw-code="${escapedRaw}"><pre><code class="hljs">${highlighted}</code></pre></div>`;
		}
	}
});

export function renderMarkdown(content: string): string {
	const raw = marked.parse(content, { async: false }) as string;
	return DOMPurify.sanitize(raw, {
		ADD_ATTR: ['data-lang', 'data-raw-code'],
		ADD_TAGS: ['code']
	});
}
