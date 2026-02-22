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
import { listenEvent } from '$lib/services/events.js';
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
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const cards = writable<CardWithStatus[]>([]);
export const cardsLoading = writable(false);
export const cardsError = writable<string | null>(null);

export const subCardsCache = writable<Map<string, CardWithStatus[]>>(new Map());

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

export const allCards = derived([cards, subCardsCache], ([$cards, $subCardsCache]) => {
	const all = [...$cards];
	for (const subs of $subCardsCache.values()) {
		all.push(...subs);
	}
	return all;
});

export async function loadCards(): Promise<void> {
	const project = get(currentProject);
	if (!project) {
		cards.set([]);
		return;
	}
	cardsLoading.set(true);
	cardsError.set(null);
	try {
		const list = await listCardsService(project.id);
		cards.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		cardsError.set(message);
		toasts.error('Failed to load cards', message);
	} finally {
		cardsLoading.set(false);
	}
}

export async function addCard(
	title: string,
	options?: {
		description?: string;
		labels?: string[];
		parentId?: string;
		statusId?: string;
	}
): Promise<CardWithStatus | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const card = await createCardService(project.id, title, options);
		await loadCards();
		toasts.success('Card created', `"${title}" has been added.`);
		return card;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to create card', message);
		return null;
	}
}

export async function updateCard(
	id: string,
	updates: { title?: string; description?: string; labels?: string[] }
): Promise<CardWithStatus | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const card = await updateCardService(project.id, id, updates);
		await loadCards();
		return card;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to update card', message);
		return null;
	}
}

export async function removeCard(id: string): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await deleteCardService(project.id, id);
		await loadCards();
		toasts.success('Card deleted', 'The card has been removed.');
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to delete card', message);
		return false;
	}
}

function findCard(id: string): CardWithStatus | undefined {
	const topLevel = get(cards).find((c) => c.id === id);
	if (topLevel) return topLevel;
	const cache = get(subCardsCache);
	for (const subs of cache.values()) {
		const found = subs.find((c) => c.id === id);
		if (found) return found;
	}
	return undefined;
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
	const card = findCard(cardId);
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
): Promise<CardWithStatus | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}

	const movingCard = findCard(id);
	const targetStatus = get(statuses).find((s) => s.id === targetStatusId);

	const runningAgentChoice = get(lastRunningAgentChoice);
	lastRunningAgentChoice.set(undefined);

	try {
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
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to move card', message);
		return null;
	}
}

export async function reorderCards(
	statusId: string,
	cardIds: string[]
): Promise<boolean> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return false;
	}
	try {
		await reorderCardsService(project.id, statusId, cardIds);
		await loadCards();
		return true;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to reorder cards', message);
		return false;
	}
}

export async function getSubCards(parentId: string): Promise<CardWithStatus[]> {
	const project = get(currentProject);
	if (!project) {
		return [];
	}
	try {
		const result = await listSubCardsService(project.id, parentId);
		subCardsCache.update((m) => {
			m.set(parentId, result);
			return new Map(m);
		});
		return result;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load sub-cards', message);
		return [];
	}
}

export async function loadSubCardsForAll(): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	const topCards = get(cards);
	const cache = new Map<string, CardWithStatus[]>();
	try {
		await Promise.all(
			topCards.map(async (card) => {
				const subs = await listSubCardsService(project.id, card.id);
				if (subs.length > 0) {
					cache.set(card.id, subs);
				}
			})
		);
		subCardsCache.set(cache);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load sub-cards', message);
	}
}

export function getCardProgress(parentId: string): CardProgress {
	const cache = get(subCardsCache);
	const subCards = cache.get(parentId) ?? [];
	const completed = subCards.filter((c) => c.status_group === 'Completed').length;
	return { completed, total: subCards.length };
}

listenEvent<{ project_id: string }>('cards-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id) {
		loadCards();
	}
});
