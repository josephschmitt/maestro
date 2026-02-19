# Phase 9: Focus Management System

**Prerequisites:** Phase 8 (card detail view)
**Goal:** Global focus management plumbing for future keyboard navigation. No shortcuts yet, just infrastructure.

## Steps

1. **Focus context store** — Tracks current focus region, current focused element, region stack (for overlays).
2. **Focus regions** — Define regions: board, card-detail, sidebar, dialog. Each region manages its own focusable elements.
3. **`use:focusable` action** — Svelte action that registers elements with the focus system, sets tabindex/aria attributes.
4. **Focus region component** — Wrapper that defines a focus region boundary.
5. **Focus ring styling** — Consistent visible focus indicators via Tailwind (`ring-2 ring-blue-500`).
6. **Tab key** — Ensure natural tab navigation works across all components.
7. **Escape key** — Closes current overlay (card detail panel pushes onto focus stack; Escape pops it and returns focus to the board card that was selected).

## Key Files to Create

```
src/lib/focus/context.ts             — Focus context store (region, element, stack)
src/lib/focus/focusable.ts           — use:focusable Svelte action
src/lib/focus/region.svelte          — Focus region wrapper component
src/lib/focus/types.ts               — FocusRegion, FocusElement types
src/lib/focus/keys.ts                — Key constants and helpers
```

## Key Files to Modify

```
src/lib/components/board/card-item.svelte              — Wrap with focusable
src/lib/components/board/status-column.svelte           — Define as focus region
src/lib/components/card-detail/card-detail-panel.svelte — Define as focus region, Escape to close
src/routes/+layout.svelte                               — Mount focus context provider
```

## Key Details

- Focus context is a Svelte writable store: `{ activeRegion: string, stack: FocusRegion[], elementsByRegion: Map<string, FocusElement[]> }`
- When a dialog/overlay opens, it pushes onto the stack. Closing pops and restores the previous region's focused element.
- `use:focusable` sets: `tabindex="0"`, `role` (if provided), registers the element with its region in the context store, adds focus/blur listeners to track the active element
- Focus ring: elements with `use:focusable` get a `focus-visible:ring-2 focus-visible:ring-ring` class
- Escape key listener on the focus context: if stack depth > 1, pop and restore. If depth == 1, do nothing (or blur).
- This phase is purely infrastructure — the keybindings (vim-style nav, hjkl, etc.) from the architecture doc's "Future Considerations" are NOT implemented here, just enabled
