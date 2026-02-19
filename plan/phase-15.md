# Phase 15: Implementation Mode + Worktrees

**Prerequisites:** Phase 14 (agent executor)
**Goal:** Agents run in isolated git worktrees for implementation tasks.

## Steps

1. **Git worktree creation** — Rust utility to create worktrees: `git worktree add <path> -b <branch>` in a linked repo.
2. **Branch naming** — Convention: `maestro/{card_id_short}-{slugified_title}` (e.g., `maestro/a1b2c3d4-add-auth`).
3. **Repo selection flow** — When card enters Started status:
   - 0 linked repos → prompt: "No repo linked. Link one, or skip (run in artifacts dir)?"
   - 1 repo → auto-select
   - 2+ repos → show selector dialog
4. **Branch name dialog** — Confirm/edit auto-generated branch name before worktree creation.
5. **Worktree reuse** — If card already has a worktree from a previous session, reuse it.
6. **Agent context for implementation** — Include exploration artifacts in the system prompt.
7. **Worktree preservation** — Never auto-delete on backward status transitions.
8. **UI updates** — Agent tab shows worktree path and branch name.

## Key Files to Create

```
src-tauri/src/fs/worktrees.rs                    — Git worktree creation, status, cleanup
src/lib/components/dialogs/repo-selector-dialog.svelte   — Choose linked repo
src/lib/components/dialogs/branch-name-dialog.svelte     — Confirm branch name
```

## Key Files to Modify

```
src-tauri/src/executor/spawn.rs      — Support worktree as working directory
src-tauri/src/executor/context.rs    — Include artifacts in implementation context
src/lib/stores/cards.ts              — Started transition triggers repo selection + worktree
src/lib/components/card-detail/tabs/agent-tab.svelte — Show worktree/branch info
```

## Key Details

- Worktree creation command: `git -C {repo_path} worktree add {worktree_path} -b {branch_name}`
- Worktree path: `~/.maestro/projects/{project_id}/worktrees/{card_id_short}-{branch_slug}/`
- `card_id_short` = first 8 chars of the card UUID
- Branch slug = title lowercased, non-alphanumeric replaced with hyphens, truncated to 40 chars
- If worktree already exists (card returning to Started), detect it via `AgentWorkspace.worktree_path` and reuse
- Repo selector dialog: list of linked directories that are git repos, with path and label
- Implementation context assembly: card description + all artifacts from `artifacts/{card_id}/` directory + implementation instructions from config
- Store worktree_path and branch_name on the AgentWorkspace record
- On backward transition (Started → Unstarted), worktree stays on disk. AgentWorkspace record preserved.
