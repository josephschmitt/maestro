# Maestro — Implementation Plan Overview

## What is Maestro?

A local-first kanban board for orchestrating AI coding agents. The full architecture spec lives at `/Users/josephschmitt/development/maestro/architecture.md`.

## Tech Stack

| Layer | Choice |
|-------|--------|
| UI Framework | SvelteKit |
| UI Components | shadcn-svelte (Bits UI + Tailwind) |
| Desktop Shell | Tauri v2 |
| Storage | SQLite (via rusqlite) |
| Drag & Drop | svelte-dnd-action |
| Agent Communication | Executor pattern + `maestro` CLI |

## Git Policy

**Initialize the repo in Phase 1. Every phase ends with one or more commits. No big-bang commits.**

## Phase Summary

| Phase | Name | Key Outcome |
|-------|------|-------------|
| 01 | Project Scaffolding + Git Init | Empty app opens in Tauri window |
| 02 | SQLite Data Layer | Full schema, Rust commands, TS types |
| 03 | Global Config + Project Management | Create/open/switch projects |
| 04 | Status Management | CRUD statuses within fixed groups |
| 05 | Card Data Layer | Card CRUD, store, sub-cards |
| 06 | Kanban Board — Static Rendering | Visual kanban with real cards |
| 07 | Drag and Drop | Cards move between columns |
| 08 | Card Detail View — Shell | Slide-over with fields + tab structure |
| 09 | Focus Management System | Global focus plumbing for keyboard nav |
| 10 | Open Questions | Questions tab + soft transition gate |
| 11 | Artifacts | File-based artifacts, markdown preview |
| 12 | Linked Directories | Link repos/folders to projects |
| 13 | Conversations Data Layer | Chat thread UI + message storage |
| 14 | Agent Executor — Process Spawning | Launch agents, stream output |
| 15 | Implementation Mode + Worktrees | Git worktrees, repo selection |
| 16 | Maestro CLI + IPC | Structured agent ↔ app communication |
| 17 | Process Re-attachment + Crash Recovery | Reconnect to agents on restart |
| 18 | Status Transition Gates + Automation | Full transition system |
| 19 | Review Workflow | Diff view, send back, approve, PR |
| 20 | Sub-Card Board View | Mini kanban for sub-cards |
| 21 | Settings + Configuration UI | Full config management |
| 22 | Polish + Error Handling | Error handling, loading states |

## Dependency Graph

```
P01 (Scaffold + Git)
 → P02 (SQLite)
   → P03 (Config + Projects)
     → P04 (Statuses)
       → P05 (Card CRUD)
         → P06 (Board Rendering)
           ├── P07 (DnD)
           ├── P08 (Card Detail)
           │   ├── P09 (Focus Mgmt)
           │   ├── P10 (Open Questions) ← also needs P07 for gate
           │   ├── P11 (Artifacts)
           │   └── P13 (Conversations)
           ├── P12 (Linked Dirs)
           │   └── P14 (Agent Spawn)
           │       ├── P15 (Worktrees)
           │       ├── P16 (CLI + IPC)
           │       ├── P17 (Re-attachment)
           │       └── P18 (Transition Gates) ← needs P10, P12, P14, P15
           │           └── P19 (Review Workflow)
           ├── P20 (Sub-card Board) ← needs P08
           ├── P21 (Settings UI) ← needs P03
           └── P22 (Polish) ← last
```

## Verification (After Each Phase)

1. `npm run tauri dev` (or `npm run dev` for web-only) boots without errors
2. New features are manually testable
3. Changes are committed to git with descriptive messages

## Project Directory Conventions

```
~/.maestro/                           # App data root
  config.toml                         # Global settings
  projects/
    {project_id}/
      db.sqlite                       # All project data
      artifacts/{card_id}/            # Exploration artifacts
      worktrees/{card_id}-{branch}/   # Git worktrees
```

## Key Data Entities

Project, LinkedDirectory, Status (within fixed StatusGroups), Card (with sub-cards via parent_id), OpenQuestion, Conversation, ConversationMessage, AgentWorkspace, Artifact.

See `architecture.md` for full schema definitions.
