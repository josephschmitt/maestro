import type { Conversation, ConversationMessage } from '$lib/types/index.js';
import { tauriInvoke } from './db.js';

export async function createConversation(
	projectId: string,
	cardId: string,
	agentType: string
): Promise<Conversation> {
	return tauriInvoke<Conversation>('create_conversation', {
		projectId,
		cardId,
		agentType
	});
}

export async function listConversations(
	projectId: string,
	cardId: string
): Promise<Conversation[]> {
	return tauriInvoke<Conversation[]>('list_conversations', { projectId, cardId });
}

export async function createMessage(
	projectId: string,
	conversationId: string,
	role: 'user' | 'agent',
	content: string
): Promise<ConversationMessage> {
	return tauriInvoke<ConversationMessage>('create_message', {
		projectId,
		conversationId,
		role,
		content
	});
}

export async function listMessages(
	projectId: string,
	conversationId: string
): Promise<ConversationMessage[]> {
	return tauriInvoke<ConversationMessage[]>('list_messages', {
		projectId,
		conversationId
	});
}

export async function countConversationMessages(
	projectId: string,
	conversationIds: string[]
): Promise<[string, number][]> {
	return tauriInvoke<[string, number][]>('count_conversation_messages', {
		projectId,
		conversationIds
	});
}
