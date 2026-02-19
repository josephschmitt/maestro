# Phase 11: Artifacts

**Prerequisites:** Phase 8 (card detail tabs)
**Goal:** File-based artifacts attached to cards, stored on disk, registered in DB.

## Steps

1. **Artifact Tauri commands** — `create_artifact`, `read_artifact`, `update_artifact`, `delete_artifact`, `list_artifacts`.
2. **File system operations** — Create `artifacts/{card_id}/` directory on first artifact. Read/write markdown files. Delete files on artifact removal.
3. **Frontend service + store** — Artifacts per card.
4. **Artifacts tab** — List of artifacts with create button.
5. **Artifact item** — Name, type icon, modified time, created_by badge. Click to open editor.
6. **Artifact editor** — Full-width markdown editor with side-by-side live preview.
7. **Delete with confirmation** — Removes DB record + file from disk.

## Key Files to Create

```
src-tauri/src/commands/artifacts.rs              — Artifact CRUD Tauri commands
src-tauri/src/fs/mod.rs                          — FS module entry
src-tauri/src/fs/artifacts.rs                    — File system operations for artifacts
src/lib/services/artifacts.ts                    — Frontend artifact service
src/lib/stores/artifacts.ts                      — Artifacts store (per card)
src/lib/components/card-detail/tabs/artifacts-tab.svelte — Tab content
src/lib/components/card-detail/artifact-item.svelte      — Single artifact row
src/lib/components/card-detail/artifact-editor.svelte    — Markdown editor for content
```

## Key Details

- File path: `~/.maestro/projects/{project_id}/artifacts/{card_id}/{slug}.md`
- Creating: user provides a display name → system generates slug filename (lowercase, hyphens) → creates `.md` file → inserts DB record
- Reading: Tauri command reads file content from disk path, returns as string
- Editing: two-pane view — textarea on left, rendered markdown on right. Auto-save on blur or after 2s debounce.
- Artifact list: sorted by `created_at` desc. Shows file icon, name, "by user"/"by agent" badge, relative timestamp.
- Delete: confirmation dialog → remove DB record → delete file from disk. If file is already missing, just remove DB record.
- The `artifacts/` directory is only created when the first artifact is added to a card (lazy creation)
