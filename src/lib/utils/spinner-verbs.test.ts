import { describe, it, expect } from 'vitest';
import { getRandomVerb, getVerbForContext } from './spinner-verbs.js';

describe('getRandomVerb', () => {
	it('returns a string', () => {
		const verb = getRandomVerb();
		expect(typeof verb).toBe('string');
		expect(verb.length).toBeGreaterThan(0);
	});

	it('does not repeat within 10 consecutive calls', () => {
		const verbs: string[] = [];
		for (let i = 0; i < 10; i++) {
			const verb = getRandomVerb();
			expect(verbs).not.toContain(verb);
			verbs.push(verb);
		}
	});
});

describe('getVerbForContext', () => {
	const thinkingVerbs = [
		'Cogitating',
		'Pondering',
		'Ruminating',
		'Deliberating',
		'Musing',
		'Contemplating',
		'Noodling',
		'Percolating',
		'Mulling',
		'Meditating',
		'Reflecting',
		'Wondering',
		'Theorizing',
		'Hypothesizing',
		'Philosophizing',
		'Ideating',
		'Introspecting',
		'Mind-wandering',
		'Thought-bubbling',
		'Braincrunching',
		'Vibing',
		'Improvising',
		'Riffing',
		'Jamming',
		'Freestyling',
		'Daydreaming',
		'Brainstorming',
		'Doodling',
		'Sketching',
		'Dreaming',
		'Imagining',
		'Channeling',
		'Envisioning',
		'Inventing',
		'Experimenting',
		'Exploring',
		'Wandering',
		'Sparking',
		'Remixing'
	];

	const codingVerbs = [
		'Crafting',
		'Assembling',
		'Weaving',
		'Sculpting',
		'Forging',
		'Composing',
		'Orchestrating',
		'Conjuring',
		'Building',
		'Constructing',
		'Engineering',
		'Fabricating',
		'Fashioning',
		'Molding',
		'Shaping',
		'Tinkering',
		'Whittling',
		'Hammering',
		'Polishing',
		'Chiseling',
		'Compiling',
		'Refactoring',
		'Linting',
		'Optimizing',
		'Benchmarking',
		'Hashing',
		'Indexing',
		'Caching',
		'Serializing',
		'Transpiling',
		'Minifying',
		'Bundling',
		'Deploying',
		'Bootstrapping',
		'Hydrating',
		'Rendering',
		'Tokenizing',
		'Diffing',
		'Debouncing',
		'Memoizing'
	];

	it('returns verbs from thinking+creative categories for "thinking" context', () => {
		for (let i = 0; i < 20; i++) {
			const verb = getVerbForContext('thinking');
			expect(thinkingVerbs).toContain(verb);
		}
	});

	it('returns verbs from working+technical categories for "coding" context', () => {
		for (let i = 0; i < 20; i++) {
			const verb = getVerbForContext('coding');
			expect(codingVerbs).toContain(verb);
		}
	});

	it('returns a string for all contexts', () => {
		for (const ctx of ['thinking', 'coding', 'reviewing', 'exploring'] as const) {
			const verb = getVerbForContext(ctx);
			expect(typeof verb).toBe('string');
			expect(verb.length).toBeGreaterThan(0);
		}
	});
});
