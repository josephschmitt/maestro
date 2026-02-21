<script lang="ts">
	import { STATUS_GROUPS } from '$lib/types/index.js';
	import type { CardProgress, OpenQuestion } from '$lib/types/index.js';
	import {
		cards,
		cardsByStatus,
		addCard,
		getCardProgress,
		getTransitionPlanForMove,
		lastRunningAgentChoice
	} from '$lib/stores/cards.js';
	import { statuses as allStatuses } from '$lib/stores/statuses.js';
	import { statusesByGroup } from '$lib/stores/statuses.js';
	import { unresolvedCountByCard, loadUnresolvedCounts } from '$lib/stores/questions.js';
	import { listQuestions } from '$lib/services/questions.js';
	import { currentProject } from '$lib/stores/project.js';
	import { get } from 'svelte/store';
	import type { PendingMove } from '$lib/utils/dnd.js';
	import type { RunningAgentChoice } from '$lib/transitions/gates.js';
	import StatusGroupColumn from './status-group-column.svelte';
	import TransitionGateDialog from './transition-gate-dialog.svelte';
	import BackwardTransitionDialog from '$lib/components/dialogs/backward-transition-dialog.svelte';
	import EmptyState from './empty-state.svelte';
	import FocusRegion from '$lib/focus/region.svelte';

	let {
		onCardClick
	}: {
		onCardClick?: (cardId: string) => void;
	} = $props();

	let hasStatuses = $derived($allStatuses.length > 0);
	let totalCards = $derived($cards.filter((c) => c.parent_id === null).length);

	let gateDialogOpen = $state(false);
	let gateQuestions = $state<OpenQuestion[]>([]);
	let gateCardTitle = $state('');
	let gateResolve: ((proceed: boolean) => void) | null = $state(null);

	let backwardDialogOpen = $state(false);
	let backwardCardTitle = $state('');
	let backwardResolve: ((choice: RunningAgentChoice | null) => void) | null = $state(null);

	$effect(() => {
		const cardIds = $cards.filter((c) => c.parent_id === null).map((c) => c.id);
		if (cardIds.length > 0) {
			loadUnresolvedCounts(cardIds);
		}
	});

	function getProgress(cardId: string): CardProgress | null {
		const progress = getCardProgress(cardId);
		if (progress.total === 0) return null;
		return progress;
	}

	function getUnresolvedCount(cardId: string): number {
		return $unresolvedCountByCard.get(cardId) ?? 0;
	}

	function showRunningAgentPrompt(cardTitle: string): Promise<RunningAgentChoice | null> {
		return new Promise((resolve) => {
			backwardCardTitle = cardTitle;
			backwardDialogOpen = true;
			backwardResolve = resolve;
		});
	}

	function showOpenQuestionsGate(cardTitle: string, questions: OpenQuestion[]): Promise<boolean> {
		return new Promise((resolve) => {
			gateCardTitle = cardTitle;
			gateQuestions = questions;
			gateDialogOpen = true;
			gateResolve = resolve;
		});
	}

	async function gateCheck(move: PendingMove): Promise<boolean> {
		lastRunningAgentChoice.set(undefined);

		const result = await getTransitionPlanForMove(move.cardId, move.targetStatusId);
		if (!result) return true;

		const { plan, card } = result;

		if (plan.gates.length === 0) return true;

		for (const gate of plan.gates) {
			if (gate.id === 'running-agent') {
				const choice = await showRunningAgentPrompt(card.title);
				if (!choice) return false;
				lastRunningAgentChoice.set(choice);
			} else if (gate.id === 'open-questions') {
				const project = get(currentProject);
				if (!project) return true;
				const questions = await listQuestions(project.id, move.cardId);
				const unresolved = questions.filter((q) => !q.resolved_at);
				if (unresolved.length > 0) {
					const proceed = await showOpenQuestionsGate(card.title, unresolved);
					if (!proceed) return false;
				}
			}
		}

		return true;
	}

	function handleGateProceed() {
		gateDialogOpen = false;
		gateResolve?.(true);
		gateResolve = null;
	}

	function handleGateCancel() {
		gateDialogOpen = false;
		gateResolve?.(false);
		gateResolve = null;
	}

	function handleBackwardChoice(choice: RunningAgentChoice) {
		backwardDialogOpen = false;
		backwardResolve?.(choice);
		backwardResolve = null;
	}

	function handleBackwardCancel() {
		backwardDialogOpen = false;
		backwardResolve?.(null);
		backwardResolve = null;
	}

	async function handleAddCard(statusId: string, title: string) {
		await addCard(title, { statusId });
	}

	function getOriginalStatusId(cardId: string): string | undefined {
		return $cards.find((c) => c.id === cardId)?.status_id;
	}
</script>

{#if !hasStatuses}
	<EmptyState />
{:else}
	<FocusRegion region="board">
		<div class="flex flex-1 flex-col overflow-hidden">
			{#if totalCards === 0}
				<div class="flex items-center justify-center border-b border-border px-4 py-3">
					<p class="text-sm text-muted-foreground">
						No cards yet. Use the + button in any column to add your first card.
					</p>
				</div>
			{/if}
			<div class="flex flex-1 gap-3 overflow-x-auto p-4">
				{#each STATUS_GROUPS as group (group)}
					<StatusGroupColumn
						{group}
						statuses={$statusesByGroup.get(group) ?? []}
						cardsByStatus={$cardsByStatus}
						{getProgress}
						{getUnresolvedCount}
						onAddCard={handleAddCard}
						{getOriginalStatusId}
						{onCardClick}
						{gateCheck}
					/>
				{/each}
			</div>
		</div>
	</FocusRegion>
{/if}

<TransitionGateDialog
	open={gateDialogOpen}
	questions={gateQuestions}
	cardTitle={gateCardTitle}
	onproceed={handleGateProceed}
	oncancel={handleGateCancel}
/>

<BackwardTransitionDialog
	open={backwardDialogOpen}
	cardTitle={backwardCardTitle}
	onchoice={handleBackwardChoice}
	oncancel={handleBackwardCancel}
/>
