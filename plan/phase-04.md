# Phase 4: Status Management

**Prerequisites:** Phase 3 (projects with seeded statuses)
**Goal:** CRUD for user-defined statuses within fixed status groups. Settings UI for managing them.

## Steps

1. **Status CRUD Tauri commands** — `create_status`, `update_status`, `delete_status`, `reorder_statuses`, `list_statuses`.
2. **Validation rules** — Cannot delete last status in a group. Cannot delete a status that has cards assigned. Cannot have more than one `is_default` per group.
3. **Svelte store** — Statuses grouped by StatusGroup in canonical order (Backlog → Cancelled). Reactive to project changes.
4. **Status management UI** — Settings panel showing groups as sections, statuses within each as reorderable items.
5. **Status badge component** — Colored badge using group color families. Reused across board and card detail.

## Key Files to Create

```
src-tauri/src/commands/statuses.rs   — Status CRUD Tauri commands
src/lib/services/statuses.ts         — Frontend status service
src/lib/stores/statuses.ts           — Statuses store (grouped by StatusGroup)
src/lib/components/status-manager.svelte  — Settings panel for managing statuses
src/lib/components/status-badge.svelte    — Colored badge component
src/routes/settings/+page.svelte     — Project settings page
```

## Key Details

- Statuses are scoped to a project (`project_id` FK)
- `sort_order` is an integer within a group; reordering recalculates all sort_order values in the group
- StatusGroup order is hardcoded: `['Backlog', 'Unstarted', 'Started', 'Completed', 'Cancelled']`
- Color families per group: Backlog=gray, Unstarted=blue, Started=yellow/amber, Completed=green, Cancelled=red
- The store provides a derived `statusesByGroup: Map<StatusGroup, Status[]>` sorted by sort_order
- Status badge shows the status name with the group's color as background/text
- Deletion checks: query `SELECT COUNT(*) FROM cards WHERE status_id = ?` before allowing delete
