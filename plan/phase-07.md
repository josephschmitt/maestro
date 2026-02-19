# Phase 7: Drag and Drop

**Prerequisites:** Phase 6 (board rendering)
**Goal:** Enable card drag-and-drop between columns using `svelte-dnd-action`.

## Steps

1. **Install `svelte-dnd-action`** — Add to package.json.
2. **Apply `use:dndzone`** on each status column's card list.
3. **Handle `consider` event** — Show drag preview, highlight drop zones.
4. **Handle `finalize` event** — Determine if card moved columns (status change) or just reordered; persist to DB.
5. **Add drag handle** — Grip icon on the left side of card items (not the whole card, so text stays selectable).
6. **Visual feedback** — Placeholder in original position, highlighted border on target column.
7. **Animated transitions** — Use `flipDurationMs` for smooth movement.

## Key Files to Create

```
src/lib/utils/dnd.ts                 — DnD utilities: transform events to card operations
```

## Key Files to Modify

```
src/lib/components/board/status-column.svelte  — Add use:dndzone directive
src/lib/components/board/card-item.svelte       — Add drag handle, drag state styling
src/lib/stores/cards.ts                         — Add moveCard/reorderCards persistence
```

## Key Details

- `use:dndzone={{ items: cards, type: 'card', flipDurationMs: 200 }}` on the card list container
- Each card item needs an `id` field for svelte-dnd-action to track
- `consider` event: update local state only (no DB write), show the preview
- `finalize` event: compare card's new `status_id` vs original. If changed, call `moveCard`. If same column, call `reorderCards`.
- Drag handle: a `⠿` (grip dots) icon on the left side of each card. Only this handle initiates drag.
- Drop zone highlight: apply a `border-dashed border-primary` class on the target column during dragover
- Cards should not be draggable when in editing mode (e.g., inline title edit)
- The DnD utility function `handleFinalize(event, statusId)` determines the operation type and dispatches the correct store action
