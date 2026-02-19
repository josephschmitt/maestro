# Phase 6: Kanban Board — Static Rendering

**Prerequisites:** Phase 5 (card data layer)
**Goal:** Render the kanban board with real data from the database. No drag-and-drop yet.

## Steps

1. **Board page** — Main route at `/board` (or root `/`). Loads cards and statuses for the current project.
2. **Board container** — Horizontal scroll layout with status group columns.
3. **Status group columns** — Each group (Backlog → Cancelled) gets a visual column container. Within each group, individual status columns are rendered.
4. **Status columns** — Header with status name + card count, vertical list of cards.
5. **Card items** — Compact card rendering: title, label pills (up to 3), sub-card progress bar (if has children), open question count (placeholder for now).
6. **Add card inline** — Text input at bottom of each column. Enter to create, Escape to cancel.
7. **Empty state** — Shown when the project has no cards at all.
8. **Accessibility** — All interactive elements get `tabindex` and `role` attributes.

## Key Files to Create

```
src/routes/board/+page.svelte                    — Main board page
src/lib/components/board/board.svelte             — Board container (horizontal scroll)
src/lib/components/board/status-group-column.svelte — Status group container
src/lib/components/board/status-column.svelte     — Single status column
src/lib/components/board/card-item.svelte         — Card on the board
src/lib/components/board/add-card-inline.svelte   — Quick card creation input
src/lib/components/board/empty-state.svelte       — No-cards empty state
```

## Key Files to Modify

```
src/routes/+layout.svelte            — Navigation to board route
```

## Key Details

- Board layout: `display: flex` with `overflow-x: auto` for horizontal scrolling
- Status groups are visually separated with subtle dividers or background color differences
- Each status column header shows: status badge (colored by group) + card count
- Card item is compact: ~60-80px height, showing title (truncated), up to 3 label pills, sub-card progress if applicable
- Label pills: small colored tags (use a hash of label text for consistent colors)
- Sub-card progress: thin progress bar at bottom of card, green fill = completed/total
- Add card inline: a "+" button at column bottom that expands to a text input. Creating a card adds it to that column's status.
- All card items have `role="button"` and `tabindex="0"` — clicking/Enter will open detail view (wired in Phase 8)
- The board subscribes to the cards store and re-renders reactively on any change
