# Phase 18: Status Transition Gates + Automation

**Prerequisites:** Phase 10 (questions), Phase 12 (directories), Phase 14 (agent), Phase 15 (worktrees)
**Goal:** Full transition system with gates, prompts, and automated actions for all status group changes.

## Steps

1. **Transition engine** — Pure function: `getTransitionPlan(card, fromStatus, toStatus) → { gates: Gate[], actions: Action[] }`.
2. **Gate definitions** — Open questions (soft), linked dirs (soft), running agents (prompt with choices).
3. **Action definitions** — Create workspace, create worktree, archive old workspace.
4. **Wire all transitions** per architecture doc:
   - Backlog → Unstarted: linked dir prompt
   - Unstarted → Started: repo selection + worktree + workspace auto-creation
   - In Progress → In Review: triggered by agent completion signal
   - In Review → In Progress: send back with feedback
   - Any backward move with running agent: stop/keep/cancel prompt
   - Completed → earlier: archive old workspace, new one on next Start
   - Cancelled → earlier: same as Completed reopen
5. **Integrate into card store** — `moveCard` calls the transition engine, renders gates, executes actions.
6. **Visual animation** — Card movement on the board gets a brief animation.

## Key Files to Create

```
src/lib/transitions/engine.ts        — Transition rule engine
src/lib/transitions/gates.ts         — Gate definitions + checkers
src/lib/transitions/actions.ts       — Post-transition action handlers
src/lib/components/dialogs/backward-transition-dialog.svelte — Running agent prompt
```

## Key Files to Modify

```
src/lib/stores/cards.ts              — Integrate transition engine into moveCard
src/lib/components/board/status-column.svelte — Trigger transition on DnD drop
```

## Transition Matrix

| From → To | Gates | Actions |
|-----------|-------|---------|
| Backlog → Unstarted | Linked dirs check (soft) | None |
| Unstarted → Started | Open questions check (soft) | Repo selection, worktree creation, workspace auto-attach |
| In Progress → In Review | None | Auto-triggered by agent exit |
| In Review → In Progress | None | Feedback message added to agent context |
| Any → Backlog/Unstarted (backward) | Running agent prompt (stop/keep/cancel) | Workspace paused if stopped. Worktree preserved. |
| Completed → any earlier | None | Archive old workspaces. New workspace on next Start. |
| Cancelled → any earlier | None | Same as Completed reopen. |

## Key Details

- Transition engine is a pure function — no side effects. Returns a plan.
- The UI layer interprets the plan: renders dialogs for gates, executes actions on approval.
- Gates have types: `soft` (warning, can override), `prompt` (requires choice), `hard` (blocks — none in MVP)
- Actions are async functions: `createWorkspace(card)`, `createWorktree(card, repo)`, `archiveWorkspace(workspace)`
- The engine is decoupled from the board UI — it could be reused by alternate views (list, table) in the future
- Artifacts and conversations are NEVER deleted on status changes
- Worktrees are NEVER auto-deleted on backward moves
