# Maestro

The conductor of the orchestra. A local-first kanban board for orchestrating AI coding agents.

## What is Maestro?

Maestro is an **orchestration layer** for AI coding agents. It doesn't provide AI — it provides a control surface for local agents like Claude Code, Codex, and Opencode. The app never holds API keys or manages subscriptions. Bring your own agent.

The user's workflow has two phases:

1. **Exploration** — Brainstorming, research, and planning with AI assistance. Happens before code exists.
2. **Implementation** — AI agents execute tasks against a codebase in isolated git worktrees.

Both phases use the same mechanism: launch an agent process, stream its output, capture artifacts.

## Tech Stack

| Layer | Choice |
|-------|--------|
| UI Framework | SvelteKit |
| UI Components | shadcn-svelte |
| Desktop Shell | Tauri v2 |
| Storage | SQLite |
| Drag & Drop | svelte-dnd-action |
| Agent Communication | Executor pattern + `maestro` CLI |

## Key Features

- **Kanban board** with fixed status groups (Backlog → Cancelled) and user-defined statuses within each
- **Cards** as the fundamental unit — each with conversations, open questions, artifacts, and agent workspaces
- **Sub-cards** for breaking down work, shown as progress on the parent
- **Agent executor** that spawns any CLI agent as a child process with streaming I/O
- **Git worktrees** for isolated implementation — each card gets its own branch and working directory
- **`maestro` CLI** for structured agent → app communication (open questions, artifacts, status changes)
- **Review workflow** with diff view, file tree, and send-back/approve/create-PR actions
- **Soft transition gates** that warn about unresolved questions before implementation begins
- **Process lifecycle management** — re-attach to running agents on app restart

## Design Principles

- **Local only** — Single user, no cloud, no auth
- **Agent agnostic** — Works with any agent that runs as a CLI process
- **Orchestration, not AI** — Maestro manages the workflow; agents do the thinking
- **Keyboard-first ready** — Focus management system built in from day one

## Project Structure

```
architecture.md          # Full architecture specification
plan/
  00-overview.md         # Implementation plan overview
  phase-01.md            # Phase 1: Project Scaffolding
  ...
  phase-22.md            # Phase 22: Polish + Error Handling
```

## License

[MIT](LICENSE)
