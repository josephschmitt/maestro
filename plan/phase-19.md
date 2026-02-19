# Phase 19: Review Workflow

**Prerequisites:** Phase 18 (transition gates)
**Goal:** Review tab with diff view, file tree, review conversation, and approve/send-back/PR actions.

## Steps

1. **Git diff commands** — Tauri commands to get file tree of changes and per-file diffs from the worktree.
2. **Diff parsing** — Parse unified diff format into structured data (hunks, lines, add/remove/context).
3. **Review tab** — Visible when card is in a review status (Started group, "In Review" or similar).
4. **File tree** — Tree view of changed files with status icons (added/modified/deleted).
5. **Diff view** — Inline diff viewer per file. Green for added lines, red for removed, gray for context.
6. **Review conversation** — Separate chat thread for review comments (distinct from exploration conversations).
7. **Send Back** — Moves card to In Progress. Review feedback included in agent's context for the next session.
8. **Approve** — Moves card to Completed.
9. **Create PR** — Pushes branch to remote, constructs PR (via `gh pr create` if available, or URL).
10. **Review count** — Tracks how many review iterations have occurred.

## Key Files to Create

```
src-tauri/src/commands/review.rs     — get_changed_files, get_file_diff, create_pr
src-tauri/src/fs/diff.rs             — Git diff parsing utilities
src/lib/services/review.ts           — Frontend review service
src/lib/stores/review.ts             — Review state store
src/lib/components/card-detail/tabs/review-tab.svelte    — Review tab container
src/lib/components/review/file-tree.svelte               — Changed files tree
src/lib/components/review/diff-view.svelte               — Inline diff viewer
src/lib/components/review/diff-line.svelte               — Single diff line
src/lib/components/review/review-actions.svelte          — Send Back / Approve / Create PR
src/lib/components/review/review-conversation.svelte     — Review chat thread
```

## Key Details

- **File tree:** Run `git -C {worktree} diff --name-status {base_branch}` → parse output into `{ path, status: 'A'|'M'|'D' }[]`
- **Per-file diff:** Run `git -C {worktree} diff {base_branch} -- {file_path}` → parse unified diff format
- **Diff rendering:** Each line gets a type (added/removed/context) and line numbers (old/new). Added = green bg, removed = red bg, context = no bg. Line numbers in gutter.
- **File tree UI:** Collapsible tree, click a file to show its diff. Icons: green "+" for added, yellow "~" for modified, red "-" for deleted.
- **Review conversation:** A Conversation record with `agent_type = "review"`. Messages from user are review comments; messages from agent are responses.
- **Send Back:** Creates a message in the review conversation with the user's feedback text, sets card status to "In Progress", increments `review_count`. Next agent session includes the review conversation in its context.
- **Approve:** Sets card status to Completed. Workspace status set to "completed".
- **Create PR:** `git -C {worktree} push -u origin {branch}`, then attempt `gh pr create --title "{card_title}" --body "{card_description}"`. Fall back to showing the URL for manual PR creation.
- Tab only visible when the card's current status belongs to a Started group and the status name suggests review (or any Started status — let the user control via status naming).
