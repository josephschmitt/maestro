import { writable, derived, get } from 'svelte/store';
import type { CardWithStatus, CardProgress } from '$lib/types/index.js';
import {
	listCards as listCardsService,
	listSubCards as listSubCardsService,
	createCard as createCardService,
	updateCard as updateCardService,
	deleteCard as deleteCardService,
	moveCard as moveCardService,
	reorderCards as reorderCardsService
} from '$lib/services/cards.js';
import {
	listWorkspaces as listWorkspacesService,
	stopAgent as stopAgentService,
	archiveCardWorkspaces as archiveCardWorkspacesService
} from '$lib/services/agent.js';
import { currentProject } from './project.js';
import { statuses } from './statuses.js';
import { linkedDirectories } from './directories.js';
import { runWorktreeFlow, type WorktreeFlowResult } from './worktree-flow.js';
import { listQuestions } from '$lib/services/questions.js';
import {
	getTransitionPlan,
	gatherTransitionContext,
	executeActions
} from '$lib/transitions/index.js';
import type { TransitionPlan } from '$lib/transitions/index.js';
import type { RunningAgentChoice, GateDataSources } from '$lib/transitions/index.js';

export const cards = writable<CardWithStatus[]>([]);

export const showLinkDirectoryPrompt = writable(false);

export const pendingWorktree = writable<Map<string, WorktreeFlowResult>>(new Map());

export const lastRunningAgentChoice = writable<RunningAgentChoice | undefined>(undefined);

export const cardsByStatus = derived(cards, ($cards) => {
	const map = new Map<string, CardWithStatus[]>();
	for (const card of $cards) {
		const list = map.get(card.status_id) ?? [];
		list.push(card);
		map.set(card.status_id, list);
	}
	for (const [key, list] of map) {
		map.set(
			key,
			list.sort((a, b) => a.sort_order - b.sort_order)
		);
	}
	return map;
});

export async function loadCards(): Promise<void> {
	const project = get(currentProject);
	if (!project) {
		cards.set([]);
		return;
	}
	const list = await listCardsService(project.id);
	cards.set(list);
}

export async function addCard(
	title: string,
	options?: {
		description?: string;
		labels?: string[];
		parentId?: string;
		statusId?: string;
	}
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const card = await createCardService(project.id, title, options);
	await loadCards();
	return card;
}

export async function updateCard(
	id: string,
	updates: { title?: string; description?: string; labels?: string[] }
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	const card = await updateCardService(project.id, id, updates);
	await loadCards();
	return card;
}

export async function removeCard(id: string): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await deleteCardService(project.id, id);
	await loadCards();
}

function getGateDataSources(): GateDataSources {
	const project = get(currentProject);
	return {
		getUnresolvedQuestions: async (cardId: string) => {
			if (!project) return [];
			const questions = await listQuestions(project.id, cardId);
			return questions.filter((q) => !q.resolved_at);
		},
		getLinkedDirCount: () => get(linkedDirectories).length,
		getRunningWorkspaceForCard: async (cardId: string) => {
			if (!project) return false;
			const workspaces = await listWorkspacesService(project.id, cardId);
			return workspaces.some((w) => w.status === 'running');
		}
	};
}

export async function getTransitionPlanForMove(
	cardId: string,
	targetStatusId: string
): Promise<{ plan: TransitionPlan; card: CardWithStatus } | null> {
	const card = get(cards).find((c) => c.id === cardId);
	if (!card) return null;

	const targetStatus = get(statuses).find((s) => s.id === targetStatusId);
	if (!targetStatus) return null;

	const ctx = await gatherTransitionContext(
		card.status_group,
		targetStatus.group,
		cardId,
		getGateDataSources()
	);

	const plan = getTransitionPlan(ctx);
	return { plan, card };
}

export async function moveCard(
	id: string,
	targetStatusId: string,
	targetSortOrder: number
): Promise<CardWithStatus> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');

	const movingCard = get(cards).find((c) => c.id === id);
	const targetStatus = get(statuses).find((s) => s.id === targetStatusId);

	const runningAgentChoice = get(lastRunningAgentChoice);
	lastRunningAgentChoice.set(undefined);

	const card = await moveCardService(project.id, id, targetStatusId, targetSortOrder);
	await loadCards();

	if (movingCard && targetStatus) {
		const ctx = await gatherTransitionContext(
			movingCard.status_group,
			targetStatus.group,
			id,
			getGateDataSources()
		);
		const plan = getTransitionPlan(ctx);

		const worktreeResult = await executeActions(plan.actions, {
			cardId: id,
			cardTitle: movingCard.title,
			showLinkDirectoryPrompt: () => showLinkDirectoryPrompt.set(true),
			runWorktreeFlow: async (cId, cTitle) => runWorktreeFlow(cId, cTitle),
			archiveWorkspaces: async (cId) => {
				await archiveCardWorkspacesService(project.id, cId);
			},
			stopAgent: async (cId) => {
				const workspaces = await listWorkspacesService(project.id, cId);
				const running = workspaces.find((w) => w.status === 'running');
				if (running) {
					await stopAgentService(project.id, running.id);
				}
			}
		}, runningAgentChoice);

		if (worktreeResult) {
			pendingWorktree.update((m) => {
				m.set(id, worktreeResult);
				return new Map(m);
			});
		}
	}

	return card;
}

export async function reorderCards(
	statusId: string,
	cardIds: string[]
): Promise<void> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	await reorderCardsService(project.id, statusId, cardIds);
	await loadCards();
}

export async function getSubCards(parentId: string): Promise<CardWithStatus[]> {
	const project = get(currentProject);
	if (!project) throw new Error('No project selected');
	return listSubCardsService(project.id, parentId);
}

export function getCardProgress(parentId: string): CardProgress {
	const $cards = get(cards);
	const subCards = $cards.filter((c) => c.parent_id === parentId);
	const completed = subCards.filter((c) => c.status_group === 'Completed').length;
	return { completed, total: subCards.length };
}
