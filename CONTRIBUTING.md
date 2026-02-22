# Contributing to Maestro

This guide covers everything you need to develop, test, and contribute to Maestro.

## Development Setup

### Prerequisites

- **Node.js** (v20+)
- **Rust** (stable toolchain via rustup)
- **pkg-config** (for native dependencies on macOS/Linux)

### Using Devbox (Recommended)

[Devbox](https://www.jetify.com/devbox) provides a reproducible development environment:

```bash
# Install devbox if you haven't
curl -fsSL https://get.jetify.com/devbox | bash

# Enter the dev environment
devbox shell
```

This automatically installs Node.js, Rust, and pkg-config.

### Manual Setup

1. Install Node.js v20+ from [nodejs.org](https://nodejs.org/)
2. Install Rust via [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```
3. Install pkg-config:
   - macOS: `brew install pkg-config`
   - Ubuntu/Debian: `apt install pkg-config`

### Install Dependencies

```bash
git clone https://github.com/your-org/maestro.git
cd maestro
npm install
```

---

## Running the App

### Browser Mode (UI Development)

```bash
npm run dev
```

Opens at `http://localhost:5173`. This mode:

- Uses **mock backends** for all Tauri commands
- Allows rapid UI iteration without compiling Rust
- Persists data in browser localStorage

Use browser mode when working on frontend components, styling, or UI logic.

### Desktop Mode (Full App)

```bash
npm run tauri dev
```

This mode:

- Compiles and runs the full Rust backend
- Uses SQLite for real data persistence
- Enables all features (agent execution, git worktrees, IPC)

Use desktop mode when working on backend features or testing end-to-end.

---

## Testing

### Frontend Tests

```bash
npm run test        # Run once
npm run test:watch  # Watch mode
```

Tests use Vitest. Test files live alongside source files with `*.test.ts` naming.

**What to test:**
- Stores and reactive logic
- Service functions
- Utility functions
- Transition engine logic
- Config resolution

**Don't test:**
- Simple Svelte component rendering (unless behavior is complex)
- Direct Tauri command calls (test via manual desktop mode)

### Backend Tests

```bash
cd src-tauri
cargo test
```

### Manual Testing Checklist

Before submitting a PR:

1. Run `npm run test` — all frontend tests pass
2. Run `npm run tauri dev` — app launches without errors
3. Create a project, card, and launch an agent
4. Verify your specific changes work as expected

---

## Building for Production

```bash
npm run tauri build
```

Output location: `src-tauri/target/release/bundle/`

Platform-specific builds:
- **macOS**: `.app` bundle and `.dmg` installer
- **Linux**: `.deb`, `.AppImage`
- **Windows**: `.exe` installer

---

## Code Conventions

### TypeScript / Svelte

- **Use TypeScript everywhere.** No `any` types.
- **Components** use `.svelte` extension. Logic files use `.ts`.
- **State management** via Svelte stores (`writable`, `derived`). Stores live in `src/lib/stores/`.
- **Services** in `src/lib/services/` wrap `invoke()` calls to Tauri commands with proper typing.
- **Use shadcn-svelte components** where available (Button, Input, Dialog, Select, Tabs, etc.). Don't reinvent primitives.
- **Tailwind for all styling.** No custom CSS files beyond `app.css`.
- **Accessibility**: All interactive elements must have `tabindex` and appropriate `role`/`aria-*` attributes.

### Rust / Tauri

- **Tauri commands** are `#[tauri::command]` async functions in `src-tauri/src/commands/`.
- **Use serde** for serialization between Rust and TypeScript.
- **UUIDs** for all entity IDs (`uuid::Uuid::new_v4()`).
- **Timestamps** as ISO-8601 strings.
- **SQLite**: `PRAGMA foreign_keys = ON` on every connection.
- **Database operations** use `rusqlite` directly (no ORM).
- **Error handling**: Return `Result<T, String>` from Tauri commands. Map errors to user-friendly messages on the frontend.

### General

- Keep changes focused. Don't refactor code unrelated to the current task.
- Prefer editing existing files over creating new ones.
- Don't add comments unless the logic is non-obvious.
- Don't over-engineer. Build what's needed, nothing more.

---

## Mock Backend System

When running in browser mode (`npm run dev`), Tauri commands are unavailable. The mock backend system provides fake implementations.

### Adding a New Command

When you add a new Tauri command:

1. Create the command in `src-tauri/src/commands/`
2. Register it in `src-tauri/src/lib.rs`
3. Create a service wrapper in `src/lib/services/`
4. **Add a mock handler** in `src/lib/services/mock/handlers/`
5. Register the handler in `src/lib/services/mock/index.ts`

### Mock Handler Example

```typescript
// src/lib/services/mock/handlers/my-feature.ts
import type { MockHandler } from '../types';

export const myCommandHandler: MockHandler = async (args) => {
  // Return mock data matching the real command's return type
  return { id: 'mock-id', name: args.name };
};
```

```typescript
// src/lib/services/mock/index.ts
import { myCommandHandler } from './handlers/my-feature';

const handlers: Record<string, MockHandler> = {
  // ... existing handlers
  'my_command': myCommandHandler,
};
```

---

## Architecture Overview

See [`architecture.md`](architecture.md) for the full specification, including:

- Data model and database schema
- Status groups and transition engine
- Agent executor lifecycle
- IPC protocol between CLI and app
- Configuration resolution order

### Key Concepts

**Status Groups**: Fixed enum (`Backlog | Unstarted | Started | Completed | Cancelled`). User-defined statuses belong to a group that determines behavior.

**Agent Executor**: Resolves config → assembles context → spawns process → streams output → detects exit. Working directories:
- **Exploration**: `~/.maestro/projects/{project_id}/artifacts/{card_id}/`
- **Implementation**: `~/.maestro/projects/{project_id}/worktrees/{card_id}-{branch}/`

**Config Resolution**: Settings resolve in order: project status override → project default → global status override → global default.

---

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
      mock/                   # Mock handlers for browser mode
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
    http/                     # HTTP server for remote access
cli/                          # `maestro` CLI binary (Rust, clap)
```

---

## Submitting Changes

1. Create a branch from `main`
2. Make your changes
3. Run tests: `npm run test`
4. Test in desktop mode: `npm run tauri dev`
5. Commit with a descriptive message
6. Open a pull request

### Commit Message Format

Keep messages concise and descriptive:

```
feat: Add card archiving
fix: Resolve worktree cleanup race condition
refactor: Extract transition engine to separate module
docs: Update CLI command reference
```
