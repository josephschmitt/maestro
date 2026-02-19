# Phase 10: Open Questions

**Prerequisites:** Phase 7 (DnD for gate), Phase 8 (card detail tabs)
**Goal:** Full open questions feature with soft transition gate.

## Steps

1. **Open Question CRUD Tauri commands** — `create_question`, `resolve_question`, `unresolve_question`, `list_questions`, `delete_question`.
2. **Frontend service + store** — Questions per card, unresolved count.
3. **Open Questions tab** — List of questions with create form, resolve/unresolve toggle.
4. **Question item component** — Question text, source badge ("user"/"agent"), resolution status, resolution text.
5. **Board badge** — Unresolved question count shown on card items.
6. **Transition gate** — When dragging a card to a Started status, check for unresolved questions. If any exist, show a warning dialog listing them with "Proceed Anyway" and "Cancel" buttons.

## Key Files to Create

```
src-tauri/src/commands/questions.rs                       — Question CRUD Tauri commands
src/lib/services/questions.ts                             — Frontend question service
src/lib/stores/questions.ts                               — Questions store (per card)
src/lib/components/card-detail/tabs/open-questions-tab.svelte — Tab content
src/lib/components/card-detail/question-item.svelte       — Single question row
src/lib/components/board/transition-gate-dialog.svelte    — Warning dialog for gates
```

## Key Files to Modify

```
src/lib/components/board/card-item.svelte     — Show open question count badge
src/lib/stores/cards.ts                       — Add transition gate check in moveCard
```

## Key Details

- Question item UI: checkbox-like toggle for resolved/unresolved, question text, small source badge, resolution text (shown when resolved, dimmed style)
- Resolved questions are collapsed by default — toggle to show them
- Create question: simple text input + "Add" button at top of tab
- Unresolved count badge on card item: small circle with number (e.g., "3") in amber/orange color, only shown when count > 0
- Transition gate logic:
  1. In the cards store `moveCard`, check target status's group
  2. If target group is "Started", query unresolved questions for the card
  3. If count > 0, don't move immediately — emit an event/callback to show the dialog
  4. Dialog lists the questions and offers "Proceed Anyway" (completes the move) or "Cancel" (aborts)
- Gate is **soft** — it never blocks, only warns
