const verbs = {
	thinking: [
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
		'Braincrunching'
	],
	working: [
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
		'Chiseling'
	],
	analyzing: [
		'Parsing',
		'Dissecting',
		'Deciphering',
		'Unraveling',
		'Sifting',
		'Scrutinizing',
		'Surveying',
		'Inspecting',
		'Examining',
		'Investigating',
		'Probing',
		'Diagnosing',
		'Evaluating',
		'Assessing',
		'Auditing',
		'Cross-referencing',
		'Pattern-matching',
		'Deep-diving',
		'Triangulating',
		'Cataloguing'
	],
	creative: [
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
		'Remixing',
		'Improvising'
	],
	technical: [
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
	]
} as const;

export type VerbCategory = keyof typeof verbs;

const contextToCategories: Record<string, VerbCategory[]> = {
	thinking: ['thinking', 'creative'],
	coding: ['working', 'technical'],
	reviewing: ['analyzing', 'technical'],
	exploring: ['creative', 'thinking', 'analyzing']
};

export type VerbContext = keyof typeof contextToCategories;

const allVerbs: string[] = Object.values(verbs).flat();
const recentVerbs: string[] = [];
const MAX_RECENT = 10;

function pickAvoidingRecent(pool: string[]): string {
	const available = pool.filter((v) => !recentVerbs.includes(v));
	const pick = available.length > 0 ? available : pool;
	const verb = pick[Math.floor(Math.random() * pick.length)];

	recentVerbs.push(verb);
	if (recentVerbs.length > MAX_RECENT) {
		recentVerbs.shift();
	}

	return verb;
}

export function getRandomVerb(): string {
	return pickAvoidingRecent(allVerbs);
}

export function getVerbForContext(context: VerbContext): string {
	const categories = contextToCategories[context];
	const pool = categories.flatMap((cat) => [...verbs[cat]]);
	return pickAvoidingRecent(pool);
}
