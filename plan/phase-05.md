# Phase 5: Card Data Layer

**Prerequisites:** Phase 4 (statuses exist)
**Goal:** Full card CRUD operations with sub-card support. No board UI yet — data layer only.

## Steps

1. **Card CRUD Tauri commands** — `create_card`, `get_card`, `update_card`, `delete_card`, `list_cards` (by project, with status info).
2. **Card movement** — `move_card(card_id, target_status_id, target_sort_order)` updates both fields atomically.
3. **Card reordering** — `reorder_cards(status_id, card_ids_in_order)` bulk-updates sort_order.
4. **Sub-card support** — `parent_id` field, `list_sub_cards(parent_id)` query.
5. **Labels** — JSON array stored as TEXT, parsed to `string[]` in TypeScript.
6. **Svelte store** — Cards indexed by `status_id`, derived `cardsByStatus` map, `moveCard` and `reorderCards` actions that persist to DB.
7. **Frontend service** — Typed methods for all card operations.

## Key Files to Create

```
src-tauri/src/commands/cards.rs      — Card CRUD, move, reorder Tauri commands
src/lib/services/cards.ts            — Frontend card service
src/lib/stores/cards.ts              — Card store with derived views
src/lib/types/card.ts                — Card type with computed fields
```

## Key Details

- New cards auto-assign the default status of the Backlog group and append to end of sort order
- `move_card` is a single DB transaction: update `status_id` and `sort_order`, then shift sort_order of other cards in the target status
- `list_cards` returns cards with their status info joined: `SELECT c.*, s.name as status_name, s.group as status_group FROM cards c JOIN statuses s ON c.status_id = s.id WHERE c.project_id = ?`
- Sub-card queries: `SELECT * FROM cards WHERE parent_id = ? ORDER BY sort_order`
- Card store provides:
  - `cardsByStatus`: `Map<string, Card[]>` — cards grouped by status_id, sorted by sort_order
  - `getSubCards(parentId)`: derived store returning children of a card
  - `getCardProgress(parentId)`: `{ completed: number, total: number }` counting sub-cards in Completed group
- Deleting a card with sub-cards cascades (ON DELETE CASCADE in schema)
- Labels are a `string[]` in TypeScript, serialized to JSON for storage
