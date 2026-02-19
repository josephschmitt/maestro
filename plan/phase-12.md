# Phase 12: Linked Directories

**Prerequisites:** Phase 8 (card detail), Phase 4 (statuses)
**Goal:** Link external directories (git repos, doc folders) to projects.

## Steps

1. **Linked Directory CRUD Tauri commands** — `add_linked_directory`, `remove_linked_directory`, `list_linked_directories`.
2. **Git detection utility** — Check if a path is a git repo (look for `.git` directory).
3. **Native file dialog** — Use Tauri `dialog.open({ directory: true })` for folder browsing.
4. **Project settings UI** — "Linked Directories" section showing linked dirs with path, label, git badge, remove button.
5. **Link directory dialog** — Browse button → native picker, label input (defaults to dir basename), auto-detects git.
6. **Transition prompt** — When card moves from Backlog to Unstarted and project has no linked directories, show a prompt suggesting to link one.

## Key Files to Create

```
src-tauri/src/commands/directories.rs            — Linked directory CRUD commands
src-tauri/src/fs/git.rs                          — Git utilities (is_repo, branch listing)
src/lib/services/directories.ts                  — Frontend directory service
src/lib/stores/directories.ts                    — Linked directories store
src/lib/components/settings/linked-directories.svelte — Settings section
src/lib/components/dialogs/link-directory-dialog.svelte — Browse + label dialog
```

## Key Files to Modify

```
src/routes/settings/+page.svelte     — Add linked directories section
src/lib/stores/cards.ts              — Add Unstarted transition prompt
```

## Key Details

- Git detection: `std::path::Path::new(&path).join(".git").exists()` in Rust
- When adding: user clicks "Browse" → native folder picker → path auto-fills → label defaults to directory basename → user can edit label → "Add" button
- `is_repo` is auto-set based on git detection. Shown as a small git icon badge next to the path.
- Removing a linked directory only removes the DB record, never deletes the actual directory
- Transition prompt: when `moveCard` targets an Unstarted group and `linked_directories` count is 0, show dialog: "This project doesn't have any directories linked. Would you like to link one now?" with options: Browse / Skip
- Each linked directory shows its full path (truncated with tooltip) and label
