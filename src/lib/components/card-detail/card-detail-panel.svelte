<script lang="ts">
	import type { CardWithStatus, Status } from '$lib/types/index.js';
	import SlideOver from '$lib/components/ui/slide-over.svelte';
	import CardHeader from './card-header.svelte';
	import CardDescription from './card-description.svelte';
	import SubCardsList from './sub-cards-list.svelte';
	import SubCardBoard from '$lib/components/board/sub-card-board.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { getSubCards, updateCard, moveCard, addCard, loadCards } from '$lib/stores/cards.js';
	import { statuses as allStatuses } from '$lib/stores/statuses.js';
	import { onMount } from 'svelte';
	import XIcon from '@lucide/svelte/icons/x';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import OpenQuestionsTab from './tabs/open-questions-tab.svelte';
	import ArtifactsTab from './tabs/artifacts-tab.svelte';
	import ConversationsTab from './tabs/conversations-tab.svelte';
	import AgentTab from './tabs/agent-tab.svelte';
	import ReviewTab from './tabs/review-tab.svelte';

	let {
		cardId,
		cards,
		onclose,
		onnavigate
	}: {
		cardId: string;
		cards: CardWithStatus[];
		onclose: () => void;
		onnavigate: (cardId: string) => void;
	} = $props();

	let subCards = $state<CardWithStatus[]>([]);
	let activeTab = $state('conversations');
	let focusMode = $state(false);

	let card = $derived(cards.find((c) => c.id === cardId));

	async function loadSubCards() {
		try {
			subCards = await getSubCards(cardId);
		} catch {
			subCards = [];
		}
	}

	$effect(() => {
		cardId;
		focusMode = false;
		loadSubCards();
	});

	async function handleTitleChange(title: string) {
		await updateCard(cardId, { title });
	}

	async function handleDescriptionChange(description: string) {
		await updateCard(cardId, { description });
	}

	async function handleStatusChange(statusId: string) {
		if (!card) return;
		await moveCard(cardId, statusId, card.sort_order);
	}

	async function handleLabelsChange(labels: string[]) {
		await updateCard(cardId, { labels });
	}

	async function handleAddSubCard() {
		await addCard('New sub-card', { parentId: cardId });
		await loadSubCards();
	}

	function handleSubCardClick(subCardId: string) {
		onnavigate(subCardId);
	}

	async function handleReviewStatusChange() {
		await loadCards();
	}

	function enterFocusMode() {
		focusMode = true;
	}

	function exitFocusMode() {
		focusMode = false;
	}

	const tabItems = [
		{ value: 'conversations', label: 'Conversations' },
		{ value: 'questions', label: 'Open Questions' },
		{ value: 'artifacts', label: 'Artifacts' },
		{ value: 'agent', label: 'Agent' },
		{ value: 'review', label: 'Review' }
	];
</script>

<SlideOver open={true} onclose={onclose}>
	{#if card}
		{#if focusMode}
			<div class="flex h-full flex-col">
				<div class="flex items-center justify-between border-b border-border px-4 py-2">
					<button
						class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted-foreground hover:bg-muted hover:text-foreground"
						onclick={exitFocusMode}
					>
						<ArrowLeftIcon size={14} />
						Back to detail
					</button>
					<button
						class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
						onclick={onclose}
						aria-label="Close panel"
					>
						<XIcon size={18} />
					</button>
				</div>

				<SubCardBoard
					parentCard={card}
					onCardClick={handleSubCardClick}
					onboardclick={onclose}
					onparentclick={exitFocusMode}
				/>
			</div>
		{:else}
			<div class="flex h-full flex-col">
				<!-- Header with back/close buttons -->
				<div class="flex items-center justify-between border-b border-border px-4 py-2">
					<div>
						{#if card.parent_id}
							<button
								class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-sm text-muted-foreground hover:bg-muted hover:text-foreground"
								onclick={() => onnavigate(card.parent_id!)}
							>
								<ArrowLeftIcon size={14} />
								Back to parent
							</button>
						{/if}
					</div>
					<button
						class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
						onclick={onclose}
						aria-label="Close panel"
					>
						<XIcon size={18} />
					</button>
				</div>

				<!-- Scrollable content -->
				<div class="flex-1 overflow-y-auto">
					<div class="flex flex-col gap-6 p-6">
						<CardHeader
							{card}
							statuses={$allStatuses}
							ontitlechange={handleTitleChange}
							onstatuschange={handleStatusChange}
							onlabelschange={handleLabelsChange}
						/>

						<CardDescription
							description={card.description}
							onchange={handleDescriptionChange}
						/>

						<SubCardsList
							{subCards}
							onaddsubcard={handleAddSubCard}
							onsubcardclick={handleSubCardClick}
							onfocus={subCards.length > 0 ? enterFocusMode : undefined}
						/>
					</div>

					<!-- Tabs -->
					<div class="border-t border-border px-6 py-4">
						<Tabs.Root bind:value={activeTab}>
							<Tabs.List class="w-full">
								{#each tabItems as tab (tab.value)}
									<Tabs.Trigger value={tab.value}>{tab.label}</Tabs.Trigger>
								{/each}
							</Tabs.List>

							<Tabs.Content value="conversations">
								<div class="py-4">
									<ConversationsTab {cardId} />
								</div>
							</Tabs.Content>

							<Tabs.Content value="questions">
								<div class="py-4">
									<OpenQuestionsTab {cardId} />
								</div>
							</Tabs.Content>

							<Tabs.Content value="artifacts">
								<div class="py-4">
									<ArtifactsTab {cardId} />
								</div>
							</Tabs.Content>

							<Tabs.Content value="agent">
								<div class="py-4">
									<AgentTab {cardId} statusId={card.status_id} />
								</div>
							</Tabs.Content>

							<Tabs.Content value="review">
								<div class="py-4">
									<ReviewTab
										{cardId}
										cardTitle={card.title}
										cardDescription={card.description}
										statuses={$allStatuses}
										onstatuschange={handleReviewStatusChange}
									/>
								</div>
							</Tabs.Content>
						</Tabs.Root>
					</div>
				</div>
			</div>
		{/if}
	{:else}
		<div class="flex h-full items-center justify-center">
			<p class="text-sm text-muted-foreground">Card not found.</p>
		</div>
	{/if}
</SlideOver>
