import { writable, derived, get } from 'svelte/store';
import type { Conversation, ConversationMessage } from '$lib/types/index.js';
import {
	listConversations as listConversationsService,
	createConversation as createConversationService,
	listMessages as listMessagesService,
	createMessage as createMessageService,
	countConversationMessages as countMessagesService
} from '$lib/services/conversations.js';
import { listenEvent } from '$lib/services/events.js';
import { currentProject } from './project.js';
import { toasts } from './toasts.js';
import { getErrorMessage } from '$lib/utils/errors.js';

export const conversations = writable<Conversation[]>([]);
export const conversationsLoading = writable(false);
export const messages = writable<ConversationMessage[]>([]);
export const messagesLoading = writable(false);
export const activeConversationId = writable<string | null>(null);
export const messageCountByConversation = writable<Map<string, number>>(new Map());

let currentCardId: string | null = null;

export const activeConversation = derived(
	[conversations, activeConversationId],
	([$conversations, $activeConversationId]) =>
		$conversations.find((c) => c.id === $activeConversationId) ?? null
);

export async function loadConversations(cardId: string): Promise<void> {
	currentCardId = cardId;
	const project = get(currentProject);
	if (!project) return;
	conversationsLoading.set(true);
	try {
		const list = await listConversationsService(project.id, cardId);
		conversations.set(list);
		if (list.length > 0) {
			await loadMessageCounts(list.map((c) => c.id));
		} else {
			messageCountByConversation.set(new Map());
		}
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load conversations', message);
	} finally {
		conversationsLoading.set(false);
	}
}

export async function addConversation(
	cardId: string,
	agentType: string = 'manual'
): Promise<Conversation | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const conversation = await createConversationService(project.id, cardId, agentType);
		await loadConversations(cardId);
		activeConversationId.set(conversation.id);
		return conversation;
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to create conversation', message);
		return null;
	}
}

export async function loadMessages(conversationId: string): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	messagesLoading.set(true);
	try {
		const list = await listMessagesService(project.id, conversationId);
		messages.set(list);
	} catch (error) {
		const message = getErrorMessage(error);
		toasts.error('Failed to load messages', message);
	} finally {
		messagesLoading.set(false);
	}
}

export async function sendMessage(
	conversationId: string,
	content: string,
	role: 'user' | 'agent' = 'user'
): Promise<ConversationMessage | null> {
	const project = get(currentProject);
	if (!project) {
		toasts.error('No project selected', 'Please select a project first.');
		return null;
	}
	try {
		const message = await createMessageService(project.id, conversationId, role, content);
		await loadMessages(conversationId);
		return message;
	} catch (error) {
		const msg = getErrorMessage(error);
		toasts.error('Failed to send message', msg);
		return null;
	}
}

export async function loadMessageCounts(conversationIds: string[]): Promise<void> {
	const project = get(currentProject);
	if (!project) return;
	if (conversationIds.length === 0) return;
	try {
		const counts = await countMessagesService(project.id, conversationIds);
		const map = new Map<string, number>();
		for (const [convId, count] of counts) {
			map.set(convId, count);
		}
		messageCountByConversation.set(map);
	} catch (error) {
		console.error('Failed to load message counts:', error);
	}
}

listenEvent<{ project_id: string }>('conversations-changed', (payload) => {
	const project = get(currentProject);
	if (project?.id === payload.project_id && currentCardId) {
		loadConversations(currentCardId);
	}
});
