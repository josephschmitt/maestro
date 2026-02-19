# Phase 13: Conversations Data Layer

**Prerequisites:** Phase 8 (card detail tabs)
**Goal:** Store and display chat threads on cards. No agent streaming yet — manual messages only.

## Steps

1. **Conversation + Message CRUD Tauri commands** — `create_conversation`, `list_conversations`, `create_message`, `list_messages`.
2. **Frontend service + store** — Conversations per card, messages per conversation.
3. **Conversations tab** — List of conversations (left panel), active conversation view (right panel).
4. **Conversation list** — Each item shows agent type, start time, message count.
5. **Conversation view** — Scrollable message thread with role-based styling.
6. **Message bubble** — User messages right-aligned (blue), agent messages left-aligned (gray). Markdown rendered.
7. **Message input** — Multiline textarea, Cmd+Enter to send.
8. **Create new conversation** — "New Conversation" button, selects agent type.

## Key Files to Create

```
src-tauri/src/commands/conversations.rs          — Conversation + Message CRUD
src/lib/services/conversations.ts                — Frontend conversation service
src/lib/stores/conversations.ts                  — Conversations store (per card)
src/lib/components/card-detail/tabs/conversations-tab.svelte — Tab content
src/lib/components/card-detail/conversation-list.svelte      — Conversation list
src/lib/components/card-detail/conversation-view.svelte      — Message thread
src/lib/components/card-detail/message-bubble.svelte         — Single message
src/lib/components/card-detail/message-input.svelte          — Text input for sending
```

## Key Details

- Conversation list: sidebar within the tab showing all conversations on the card. Click to select active one.
- Message bubble: distinct visual styles for user (right-aligned, blue-ish bg) vs agent (left-aligned, gray bg)
- Messages render markdown content to HTML (use a markdown renderer like `marked` or `snarkdown`)
- Timestamps: stored as ISO-8601, displayed as relative time ("2 minutes ago", "yesterday")
- Message input: `<textarea>` with `rows="3"`, expands as user types. Cmd+Enter (Mac) / Ctrl+Enter sends. Shift+Enter for newline.
- Auto-scroll: conversation view scrolls to bottom on new message, maintains bottom-pinned behavior
- "New Conversation" creates a record with `agent_type` set to "manual" (or user-selected). At this phase, all messages are manually added — agent streaming comes in Phase 14.
- The conversation tab is the default/first tab in the card detail view
