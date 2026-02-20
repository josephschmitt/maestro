import type { Card, Status, Project, GlobalConfigResponse, OpenQuestion, Artifact, LinkedDirectory, Conversation, ConversationMessage, AgentWorkspace } from '$lib/types/index.js';
import type { StatusGroup } from '$lib/types/status.js';

export interface MockStore {
	projects: Project[];
	statuses: Status[];
	cards: Card[];
	questions: OpenQuestion[];
	artifacts: Artifact[];
	artifactContents: Map<string, string>;
	linkedDirectories: LinkedDirectory[];
	conversations: Conversation[];
	conversationMessages: ConversationMessage[];
	agentWorkspaces: AgentWorkspace[];
	globalConfig: GlobalConfigResponse;
}

let store: MockStore | null = null;

export function newId(): string {
	const bytes = new Uint8Array(16);
	crypto.getRandomValues(bytes);
	bytes[6] = (bytes[6] & 0x0f) | 0x40;
	bytes[8] = (bytes[8] & 0x3f) | 0x80;
	const hex = [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
	return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function nowISO(): string {
	return new Date().toISOString();
}

export function getStore(): MockStore {
	if (!store) {
		store = createSeedStore();
	}
	return store;
}

function createSeedStore(): MockStore {
	const projectId = newId();
	const now = nowISO();

	const statuses = createSeedStatuses(projectId, now);
	const cards = createSeedCards(projectId, statuses, now);

	return {
		projects: [
			{
				id: projectId,
				name: 'Demo Project',
				agent_config: {},
				base_path: null,
				created_at: now,
				updated_at: now
			}
		],
		statuses,
		cards,
		questions: createSeedQuestions(cards),
		artifacts: [],
		artifactContents: new Map(),
		linkedDirectories: [],
		conversations: [],
		conversationMessages: [],
		agentWorkspaces: [],
		globalConfig: {
			storage_base_path: '/tmp/maestro',
			default_agent: 'claude-code',
			last_project_id: projectId,
			agents: [
				{
					name: 'claude-code',
					binary: 'claude',
					flags: ['--dangerously-skip-permissions'],
					custom_command: null
				}
			]
		}
	};
}

function createSeedStatuses(projectId: string, now: string): Status[] {
	const defs: { group: StatusGroup; name: string; isDefault: boolean }[] = [
		{ group: 'Backlog', name: 'Icebox', isDefault: true },
		{ group: 'Unstarted', name: 'To Do', isDefault: true },
		{ group: 'Started', name: 'In Progress', isDefault: true },
		{ group: 'Started', name: 'In Review', isDefault: false },
		{ group: 'Completed', name: 'Done', isDefault: true },
		{ group: 'Cancelled', name: 'Cancelled', isDefault: true }
	];

	return defs.map((d, i) => ({
		id: newId(),
		project_id: projectId,
		group: d.group,
		name: d.name,
		sort_order: i,
		is_default: d.isDefault,
		created_at: now
	}));
}

function createSeedCards(projectId: string, statuses: Status[], now: string): Card[] {
	const statusByName = (name: string) => statuses.find((s) => s.name === name)!;

	const defs: { title: string; status: string; description: string; labels: string[] }[] = [
		{
			title: 'Set up project scaffolding',
			status: 'Done',
			description: 'Initialize the monorepo with Tauri, SvelteKit, and SQLite.',
			labels: ['setup']
		},
		{
			title: 'Design database schema',
			status: 'Done',
			description: 'Define tables for projects, cards, statuses, and related entities.',
			labels: ['database', 'design']
		},
		{
			title: 'Implement drag and drop',
			status: 'In Review',
			description:
				'Add drag-and-drop support for moving cards between columns and reordering within columns.',
			labels: ['feature', 'ui']
		},
		{
			title: 'Build card detail panel',
			status: 'In Progress',
			description:
				'Create the slide-out panel that shows card details, description, and sub-cards.',
			labels: ['feature', 'ui']
		},
		{
			title: 'Add agent executor',
			status: 'To Do',
			description:
				'Implement the agent spawning and lifecycle management system for running AI agents.',
			labels: ['feature', 'backend']
		},
		{
			title: 'Write CLI tool',
			status: 'To Do',
			description:
				'Build the `maestro` CLI binary for agent-to-app communication over Unix sockets.',
			labels: ['feature', 'cli']
		},
		{
			title: 'Add keyboard shortcuts',
			status: 'Icebox',
			description: 'Implement global keyboard shortcuts for common actions (new card, search).',
			labels: ['feature', 'ux']
		},
		{
			title: 'Dark mode support',
			status: 'Icebox',
			description: 'Add theme toggle and dark mode color scheme.',
			labels: ['feature', 'ui']
		}
	];

	return defs.map((d, i) => {
		const status = statusByName(d.status);
		return {
			id: newId(),
			project_id: projectId,
			parent_id: null,
			status_id: status.id,
			title: d.title,
			description: d.description,
			labels: d.labels,
			sort_order: i,
			created_at: now,
			updated_at: now
		};
	});
}

function createSeedQuestions(cards: Card[]): OpenQuestion[] {
	const inProgressCard = cards.find((c) => c.title === 'Build card detail panel');
	if (!inProgressCard) return [];
	const now = nowISO();
	return [
		{
			id: newId(),
			card_id: inProgressCard.id,
			question: 'Should the panel support keyboard navigation between tabs?',
			resolution: null,
			source: 'user',
			resolved_by: null,
			created_at: now,
			resolved_at: null
		},
		{
			id: newId(),
			card_id: inProgressCard.id,
			question: 'What animation should the slide-over use?',
			resolution: 'Use a simple slide from right with 200ms duration',
			source: 'agent',
			resolved_by: 'user',
			created_at: now,
			resolved_at: now
		}
	];
}

export function enrichCard(
	card: Card,
	statuses: Status[]
): import('$lib/types/card.js').CardWithStatus {
	const status = statuses.find((s) => s.id === card.status_id);
	return {
		...card,
		status_name: status?.name ?? 'Unknown',
		status_group: status?.group ?? 'Backlog'
	};
}
