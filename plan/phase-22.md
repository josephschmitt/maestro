# Phase 22: Polish + Error Handling

**Prerequisites:** All previous phases
**Goal:** Harden the application with error handling, loading states, notifications, and edge cases.

## Steps

1. **Global error boundary** — Catch unhandled errors, show fallback UI with error details.
2. **Toast notifications** — Success, error, warning, info. Auto-dismiss for success/info (5s), sticky for errors.
3. **Loading skeletons** — Skeleton placeholders for board, card detail, conversations while data loads.
4. **Confirmation dialogs** — Reusable dialog for destructive actions (delete card, remove directory, delete artifact).
5. **Empty states** — Meaningful empty states for every list (no conversations, no artifacts, no questions, no linked dirs).
6. **Form validation** — Required field checks, path validation, duplicate name checks across all input forms.
7. **Graceful file handling** — If an artifact file is missing from disk (deleted outside app), handle gracefully — show warning, offer to remove the DB record.
8. **DB integrity checks** — On startup, verify artifact files exist, check for orphaned records, validate foreign keys.
9. **Error wrapping** — Every `invoke()` call wrapped in try/catch with user-friendly error messages surfaced as toasts.
10. **Loading states** — All async operations show loading indicators. Stores track `loading` and `error` states.

## Key Files to Create

```
src/lib/components/ui/toast.svelte               — Toast notification component
src/lib/components/ui/toast-container.svelte      — Toast stack (positioned fixed top-right)
src/lib/stores/toasts.ts                         — Toast store (add, dismiss, auto-remove)
src/lib/components/ui/skeleton.svelte            — Loading skeleton components
src/lib/components/ui/error-boundary.svelte      — Error boundary wrapper
src/lib/components/ui/confirm-dialog.svelte      — Reusable confirmation dialog
src/lib/utils/errors.ts                          — Error types, formatting utilities
```

## Key Files to Modify

```
src/routes/+layout.svelte            — Mount toast container, error boundary
src/lib/services/*.ts                — Wrap all invoke calls with error handling
src/lib/stores/*.ts                  — Add loading/error state tracking
```

## Key Details

- **Toast system:** Store is an array of `{ id, type, title, message, duration }`. `addToast(type, title, message)` helper. Auto-dismiss via `setTimeout`. Toasts stack vertically, newest on top.
- **Error boundary:** Svelte `onError` handler in the root layout. Shows a "Something went wrong" message with error details in a collapsible section. "Reload" button.
- **Loading skeletons:** Use shadcn-svelte Skeleton component. Board shows column skeletons with card-shaped placeholders. Card detail shows field-shaped skeletons.
- **Confirm dialog:** Generic `ConfirmDialog` component: `{ title, message, confirmLabel, cancelLabel, onConfirm }`. Shown before: delete card, delete artifact, remove linked dir, stop agent.
- **Empty states:** Each list gets a centered message with an icon: "No conversations yet — start one to begin chatting with an agent", "No artifacts yet — create a planning document", etc.
- **Error formatting:** Map Rust error types to user-friendly messages. "Database error" → "Failed to save changes. Please try again." Network-like errors → "Could not connect to the agent process."
- **Startup integrity:** Query all artifacts, check file existence, flag missing ones. Log warnings for orphaned records (workspace without a card, etc.).
