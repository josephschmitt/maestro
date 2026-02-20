# Maestro — Agent Guide

## What is this project?

Maestro is a local-first kanban board for orchestrating AI coding agents. It's a Tauri v2 desktop app with a SvelteKit frontend and SQLite storage. See `architecture.md` for the full spec.

## Tech Stack

- **Frontend:** SvelteKit (TypeScript, SSR disabled, `adapter-static`)
- **UI Components:** shadcn-svelte (Bits UI primitives, Tailwind CSS)
- **Desktop Shell:** Tauri v2 (Rust backend)
- **Database:** SQLite via `rusqlite` (bundled)
- **Drag & Drop:** svelte-dnd-action
- **Agent Communication:** Unix socket IPC + `maestro` CLI binary

## Project Structure

```
src/                          # SvelteKit frontend
  routes/                     # Pages (board, settings)
  lib/
    components/               # Svelte components
      ui/                     # shadcn-svelte base components
      board/                  # Kanban board components
      card-detail/            # Card detail panel + tabs
      review/                 # Review workflow components
      settings/               # Settings UI
      dialogs/                # Modal dialogs
    stores/                   # Svelte stores (reactive state)
    services/                 # Frontend services (Tauri invoke wrappers)
    types/                    # TypeScript interfaces
    focus/                    # Focus management system
    transitions/              # Status transition engine
    utils/                    # Utility functions
src-tauri/                    # Tauri Rust backend
  src/
    commands/                 # Tauri command handlers
    db/                       # SQLite schema, migrations, connection
    executor/                 # Agent process spawning + lifecycle
    config/                   # Global + project config management
    fs/                       # File system operations (artifacts, worktrees, git)
    ipc/                      # Unix socket server for CLI communication
cli/                          # `maestro` CLI binary (Rust, clap)
plan/                         # Implementation plan (phase files)
architecture.md               # Full architecture specification
```

## Implementation Plan

The build is broken into 22 phases in `plan/`. Read `plan/00-overview.md` for the summary and dependency graph. Each `plan/phase-XX.md` contains the steps, files, and details for that phase.

When working on a phase, read the overview and that phase's file for context.

## Coding Conventions

### Svelte / TypeScript

- Use TypeScript everywhere. No `any` types.
- Components use `.svelte` extension. Logic files use `.ts`.
- State management via Svelte stores (`writable`, `derived`). Stores live in `src/lib/stores/`.
- Services in `src/lib/services/` wrap `invoke()` calls to Tauri commands with proper typing.
- Use shadcn-svelte components where available (Button, Input, Dialog, Select, Tabs, etc.). Don't reinvent primitives.
- Tailwind for all styling. No custom CSS files beyond `app.css`.
- All interactive elements must have `tabindex` and appropriate `role`/`aria-*` attributes for accessibility.

### Mock Backend

- When adding a new Tauri command and its corresponding service function, also add a mock handler in `src/lib/services/mock/handlers/`. Update the dispatch map in `mock/index.ts`. This keeps browser-mode development (`npm run dev`) working.

### Rust / Tauri

- Tauri commands are `#[tauri::command]` async functions in `src-tauri/src/commands/`.
- Use `serde` for serialization between Rust and TypeScript.
- UUIDs for all entity IDs (`uuid::Uuid::new_v4()`).
- Timestamps as ISO-8601 strings.
- `PRAGMA foreign_keys = ON` on every SQLite connection.
- Database operations use `rusqlite` directly (no ORM).
- Error handling: return `Result<T, String>` from Tauri commands. Map errors to user-friendly messages on the frontend.

### General

- Keep changes focused. Don't refactor code unrelated to the current task.
- Commit after each meaningful chunk of work with a descriptive message.
- Prefer editing existing files over creating new ones when possible.
- Don't add comments unless the logic is non-obvious.
- Don't over-engineer. Build what the phase calls for, nothing more.

## Data Model Quick Reference

Core entities (see `architecture.md` for full schema):

- **Project** — top-level container, owns everything
- **Status** — user-defined, within fixed StatusGroups (Backlog, Unstarted, Started, Completed, Cancelled)
- **Card** — fundamental unit, has `parent_id` for sub-cards
- **OpenQuestion** — planning gate, soft warning before Started
- **Conversation / ConversationMessage** — AI chat threads on cards
- **AgentWorkspace** — agent session record (PID, status, worktree, branch)
- **Artifact** — files on disk registered in DB (`artifacts/{card_id}/`)
- **LinkedDirectory** — external directories linked to a project

## Key Patterns

### Status Groups

StatusGroup is a fixed enum, not a table: `Backlog | Unstarted | Started | Completed | Cancelled`. User-defined statuses belong to a group. The group determines behavior (gates, agent mode, etc.).

### Agent Executor

All agent interaction follows: resolve config → assemble context → spawn process → stream output → detect exit. Working directory depends on mode:
- **Exploration:** `~/.maestro/projects/{project_id}/artifacts/{card_id}/`
- **Implementation:** `~/.maestro/projects/{project_id}/worktrees/{card_id}-{branch}/`

### Config Resolution

Settings resolve in order: project status override → project default → global status override → global default. See `architecture.md` Configuration section.

### Transition Engine

Card status changes go through a transition engine that returns gates (warnings/prompts) and actions (workspace creation, worktree setup). The engine is a pure function, decoupled from UI.

## Testing

- **Unit tests:** Use Vitest (`npx vitest` or `npm run test`). Write tests alongside the code as you build — don't defer testing to a later phase. Test files live next to source files using the `*.test.ts` naming convention.
- **What to test:** Stores, services, utility functions, transition engine logic, config resolution. Pure logic is the priority. Don't test Svelte component rendering unless behavior is complex.
- **Tauri commands:** Test manually via `npm run tauri dev`. Rust-side unit tests can use `cargo test` in `src-tauri/`.
- **Web UI:** `npm run dev` for browser testing (some Tauri-specific features won't work in browser).
- **Verify each phase's deliverables** before moving to the next.
