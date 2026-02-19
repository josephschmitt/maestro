# Phase 8: Card Detail View — Shell

**Prerequisites:** Phase 6 (board rendering)
**Goal:** Slide-over panel for viewing and editing card details. Tab structure for future content.

## Steps

1. **Slide-over component** — Generic reusable panel that slides in from the right (~60% width). Board remains visible but dimmed.
2. **Card detail panel** — Wraps the slide-over, loads card data, renders header + tabs.
3. **Inline title editing** — Renders as text, click to switch to input, blur or Enter to save.
4. **Description editor** — Markdown textarea with "Preview" toggle that renders markdown.
5. **Status selector** — Dropdown grouped by StatusGroup.
6. **Labels editor** — Add/remove label pills with an input field.
7. **Sub-cards section** — List of sub-cards with status badges. "Add sub-card" button.
8. **Tab bar** — Tabs for: Conversations, Open Questions, Artifacts, Agent, Review. All show placeholder "Coming soon" content.
9. **URL state** — Opening a card updates URL to `/board?card={id}`. Closing removes the param. Direct navigation to URL opens the card.

## Key Files to Create

```
src/lib/components/card-detail/card-detail-panel.svelte  — Main panel with tabs
src/lib/components/card-detail/card-header.svelte         — Title, status, labels
src/lib/components/card-detail/card-description.svelte    — Markdown editor/preview
src/lib/components/card-detail/sub-cards-list.svelte      — Sub-card list + add button
src/lib/components/card-detail/status-selector.svelte     — Status dropdown
src/lib/components/card-detail/label-editor.svelte        — Label management
src/lib/components/ui/slide-over.svelte                   — Generic slide-over panel
src/lib/components/ui/markdown-editor.svelte              — Reusable markdown editor
```

## Key Files to Modify

```
src/lib/components/board/card-item.svelte     — Click handler to open detail
src/routes/board/+page.svelte                 — Mount card detail panel, handle URL param
```

## Key Details

- Slide-over uses a backdrop overlay with `bg-black/50` and `z-50`; clicking backdrop closes panel
- Panel animates in with a CSS transition (`transform: translateX`)
- Title: inline editable — displays as `<h2>`, click transitions to `<input>`, saves on blur/Enter, cancels on Escape
- Description: textarea at ~200px height, with a toggle between "Edit" and "Preview" modes. Preview renders markdown to HTML.
- Status selector: use shadcn-svelte Select component. Options grouped by StatusGroup with group labels.
- Labels: render as pills with "×" remove button. Input field below with Enter to add new label.
- Sub-cards list: each sub-card shows title + status badge. Clickable to open that sub-card's detail (replaces current panel content).
- Tab bar: use shadcn-svelte Tabs component. Tab content is `{#if activeTab === 'conversations'}...{/if}` pattern.
- All changes auto-save on blur/change (no explicit Save button)
