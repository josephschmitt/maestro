import type { ChangedFile, FileDiff, CreatePrResult } from '$lib/types/index.js';
import { getStore, newId, nowISO } from '../store.js';

const mockChangedFiles: ChangedFile[] = [
	{ path: 'src/lib/components/board/board.svelte', status: 'M' },
	{ path: 'src/lib/stores/review.ts', status: 'A' },
	{ path: 'src/lib/services/old-service.ts', status: 'D' },
	{ path: 'src/lib/utils/helpers.ts', status: 'M' }
];

function mockFileDiff(filePath: string): FileDiff {
	return {
		path: filePath,
		hunks: [
			{
				old_start: 1,
				old_count: 5,
				new_start: 1,
				new_count: 7,
				header: '',
				lines: [
					{
						line_type: 'context',
						content: 'import { writable } from "svelte/store";',
						old_line: 1,
						new_line: 1
					},
					{
						line_type: 'context',
						content: '',
						old_line: 2,
						new_line: 2
					},
					{
						line_type: 'removed',
						content: 'const oldValue = "hello";',
						old_line: 3,
						new_line: null
					},
					{
						line_type: 'added',
						content: 'const newValue = "hello world";',
						old_line: null,
						new_line: 3
					},
					{
						line_type: 'added',
						content: 'const extra = true;',
						old_line: null,
						new_line: 4
					},
					{
						line_type: 'context',
						content: '',
						old_line: 4,
						new_line: 5
					},
					{
						line_type: 'context',
						content: 'export default {};',
						old_line: 5,
						new_line: 6
					}
				]
			}
		]
	};
}

export function get_changed_files(_args: Record<string, unknown>): ChangedFile[] {
	return mockChangedFiles;
}

export function get_file_diff(args: Record<string, unknown>): FileDiff {
	return mockFileDiff(args.filePath as string);
}

export function send_back_card(args: Record<string, unknown>): void {
	const store = getStore();
	const cardId = args.cardId as string;
	const inProgressStatusId = args.inProgressStatusId as string;
	const feedback = args.feedback as string;

	const card = store.cards.find((c) => c.id === cardId);
	if (card) {
		card.status_id = inProgressStatusId;
		card.updated_at = nowISO();
	}

	const workspace = store.agentWorkspaces.find(
		(w) => w.card_id === cardId && w.status !== 'completed' && w.status !== 'failed'
	);
	if (workspace) {
		workspace.review_count += 1;
	}

	let conv = store.conversations.find(
		(c) => c.card_id === cardId && c.agent_type === 'review'
	);
	if (!conv) {
		conv = {
			id: newId(),
			card_id: cardId,
			agent_type: 'review',
			started_at: nowISO(),
			ended_at: null
		};
		store.conversations.push(conv);
	}

	store.conversationMessages.push({
		id: newId(),
		conversation_id: conv.id,
		role: 'user',
		content: feedback,
		timestamp: nowISO()
	});
}

export function approve_card(args: Record<string, unknown>): void {
	const store = getStore();
	const cardId = args.cardId as string;
	const completedStatusId = args.completedStatusId as string;

	const card = store.cards.find((c) => c.id === cardId);
	if (card) {
		card.status_id = completedStatusId;
		card.updated_at = nowISO();
	}

	for (const ws of store.agentWorkspaces) {
		if (ws.card_id === cardId && ws.status !== 'completed' && ws.status !== 'failed') {
			ws.status = 'completed';
			ws.completed_at = nowISO();
		}
	}
}

export function create_pr(_args: Record<string, unknown>): CreatePrResult {
	return { url: 'https://github.com/example/repo/pull/1' };
}

export function get_review_count(args: Record<string, unknown>): number {
	const store = getStore();
	const cardId = args.cardId as string;
	const workspaces = store.agentWorkspaces.filter((w) => w.card_id === cardId);
	return Math.max(0, ...workspaces.map((w) => w.review_count), 0);
}
