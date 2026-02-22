# Maestro

The conductor of the orchestra. A local-first kanban board for orchestrating AI coding agents.

## Quick Start (Production)

### Prerequisites

- **Node.js** (v20+)
- **Rust** (stable toolchain)
- An AI coding agent CLI (e.g., `claude` from Claude Code)

### Build & Install

```bash
git clone https://github.com/your-org/maestro.git
cd maestro
npm install
npm run tauri build
```

The app bundle will be in `src-tauri/target/release/bundle/`.

### First Launch

1. Open Maestro
2. Create a new project (give it a name)
3. Link a directory (optional — point to your codebase)

### Configure an Agent

Maestro ships with two pre-configured agent profiles:

| Profile | Binary | Flags |
|---------|--------|-------|
| `claude-code` | `claude` | `--dangerously-skip-permissions` |
| `codex` | `codex` | `--full-auto` |

Make sure your agent binary is in your PATH. To add custom agents, go to **Settings > Agent Profiles**.

### Create Your First Card

1. Click **+ New Card** in any column
2. Give it a title and description
3. Click the card to open details
4. Go to the **Agent** tab
5. Click **Launch Agent**

The agent starts in exploration mode, working in an isolated artifacts directory.

---

## Quick Start (Development)

### Prerequisites

- **Node.js** (v20+)
- **Rust** (stable toolchain)
- **pkg-config** (for native dependencies)

Or use **Devbox** for a reproducible environment:

```bash
devbox shell
```

### Setup

```bash
git clone https://github.com/your-org/maestro.git
cd maestro
npm install
```

### Run in Browser Mode

```bash
npm run dev
```

Opens at `http://localhost:5173`. Tauri features are mocked — useful for UI development.

### Run Full Desktop App

```bash
npm run tauri dev
```

### Run Tests

```bash
npm run test
```

### Build for Production

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development documentation.

---

## What is Maestro?

Maestro is an **orchestration layer** for AI coding agents. It doesn't provide AI — it provides a control surface for local agents like Claude Code, Codex, and Opencode. The app never holds API keys or manages subscriptions. Bring your own agent.

The user's workflow has two phases:

1. **Exploration** — Brainstorming, research, and planning with AI assistance. Happens before code exists.
2. **Implementation** — AI agents execute tasks against a codebase in isolated git worktrees.

Both phases use the same mechanism: launch an agent process, stream its output, capture artifacts.

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
- **HTTP server mode** — access from iPad or other devices on your network

## Design Principles

- **Local only** — Single user, no cloud, no auth
- **Agent agnostic** — Works with any agent that runs as a CLI process
- **Orchestration, not AI** — Maestro manages the workflow; agents do the thinking
- **Keyboard-first ready** — Focus management system built in from day one

---

## Configuration

### Global Config

Located at `~/.maestro/config.toml`:

```toml
[storage]
base_path = "~/.maestro"

[defaults]
agent = "claude-code"

[agents.claude-code]
binary = "claude"
flags = ["--dangerously-skip-permissions"]

[agents.codex]
binary = "codex"
flags = ["--full-auto"]

[http_server]
bind_address = "127.0.0.1"
port = 3456
```

### Agent Profiles

Each agent profile has:

| Field | Description |
|-------|-------------|
| `binary` | The executable name (must be in PATH) |
| `flags` | Command-line flags passed to the agent |
| `custom_command` | Optional: full command override |
| `env_vars` | Optional: environment variables to set |

### Network Mode (iPad/Remote Access)

To access Maestro from another device:

1. Go to **Settings > HTTP Server**
2. Change bind address to **"All network interfaces"**
3. Note the server URL and auth token
4. Access from your device's browser

When network mode is active, all requests require the bearer token for authentication.

---

## The `maestro` CLI

Agents communicate back to Maestro via the `maestro` CLI binary. When an agent is launched, it receives environment variables:

| Variable | Description |
|----------|-------------|
| `MAESTRO_SOCKET` | Unix socket path for IPC |
| `MAESTRO_CARD_ID` | Current card's UUID |

### Commands

| Command | Purpose |
|---------|---------|
| `maestro question "..."` | Surface an open question for the user |
| `maestro resolve-question --id <id>` | Mark a question as resolved |
| `maestro add-artifact --file <path>` | Register a file as an artifact |
| `maestro set-status <status>` | Change card status (e.g., `in-review`) |
| `maestro log "..."` | Record a progress note |
| `maestro get-card` | Get current card details (JSON) |
| `maestro get-artifacts` | List card artifacts (JSON) |
| `maestro get-parent` | Get parent card if sub-card (JSON) |

See `architecture.md` for full IPC protocol details.

---

## Tech Stack

| Layer | Choice |
|-------|--------|
| UI Framework | SvelteKit |
| UI Components | shadcn-svelte |
| Desktop Shell | Tauri v2 |
| Storage | SQLite |
| Drag & Drop | svelte-dnd-action |
| Agent Communication | Unix socket IPC + `maestro` CLI |

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
    http/                     # HTTP server for remote access
cli/                          # `maestro` CLI binary (Rust, clap)
architecture.md               # Full architecture specification
```

## License

[MIT](LICENSE)
