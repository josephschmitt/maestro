import type { Conversation, ConversationMessage } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

export function create_conversation(args: Record<string, unknown>): Conversation {
	const store = getStore();
	const conversation: Conversation = {
		id: newId(),
		card_id: args.cardId as string,
		agent_type: (args.agentType as string) ?? 'manual',
		started_at: nowISO(),
		ended_at: null
	};
	store.conversations.push(conversation);
	return conversation;
}

export function list_conversations(args: Record<string, unknown>): Conversation[] {
	const store = getStore();
	return store.conversations
		.filter((c) => c.card_id === args.cardId)
		.sort((a, b) => b.started_at.localeCompare(a.started_at));
}

export function create_message(args: Record<string, unknown>): ConversationMessage {
	const store = getStore();
	const message: ConversationMessage = {
		id: newId(),
		conversation_id: args.conversationId as string,
		role: args.role as 'user' | 'agent',
		content: args.content as string,
		timestamp: nowISO()
	};
	store.conversationMessages.push(message);
	return message;
}

export function list_messages(args: Record<string, unknown>): ConversationMessage[] {
	const store = getStore();
	return store.conversationMessages
		.filter((m) => m.conversation_id === args.conversationId)
		.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
}

export function count_conversation_messages(args: Record<string, unknown>): [string, number][] {
	const store = getStore();
	const conversationIds = args.conversationIds as string[];
	return conversationIds.map((convId) => {
		const count = store.conversationMessages.filter(
			(m) => m.conversation_id === convId
		).length;
		return [convId, count];
	});
}
