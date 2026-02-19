# Phase 20: Sub-Card Board View

**Prerequisites:** Phase 8 (card detail)
**Goal:** Focus into a parent card to manage its sub-cards as a mini kanban board.

## Steps

1. **Focus action** — "Focus" button on parent cards (cards that have sub-cards). Opens sub-card board view.
2. **Sub-card board** — Same kanban layout as main board, but scoped to sub-cards of the parent.
3. **Sub-card independence** — Each sub-card has its own status, conversations, artifacts, agent workspaces.
4. **Progress summary** — Parent card on the main board shows "2/4 completed" with a thin progress bar.
5. **Breadcrumb navigation** — "Board → Parent Card Title → Sub-card Title" for navigating back up.
6. **DnD in focused view** — Sub-cards are draggable within the focused board view.

## Key Files to Create

```
src/lib/components/board/sub-card-board.svelte   — Mini kanban scoped to parent
src/lib/components/board/breadcrumbs.svelte       — Navigation breadcrumbs
```

## Key Files to Modify

```
src/lib/components/card-detail/card-detail-panel.svelte — "Focus" mode replaces detail with sub-card board
src/lib/components/board/card-item.svelte               — Progress bar for parent cards
src/lib/stores/cards.ts                                 — getSubCards(parentId), progress computation
```

## Key Details

- **Progress computation:** `completed = subCards.filter(c => statusGroup(c.status_id) === 'Completed').length`, `total = subCards.length`
- **Progress bar:** Thin bar (3px) at bottom of card item. Green fill proportional to `completed / total`. Hidden when total is 0.
- **Sub-card board:** Reuses `board.svelte`, `status-column.svelte`, `card-item.svelte` but filtered to `cards WHERE parent_id = {parentId}`
- **Focusing:** The card detail panel switches from its normal tabbed view to the sub-card board. This is a mode within the detail panel, not a separate route.
- **Breadcrumbs:** Clickable path. "Board" returns to main board. Parent card title returns to parent's detail. Sub-card titles are for orientation.
- **Sub-cards use the same project statuses** as top-level cards (no separate status set per parent)
- **Creating sub-cards:** "Add sub-card" in the focused view uses the same inline creation as the main board, but sets `parent_id`
