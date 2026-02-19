# Maestro — Architecture v1

*The conductor of the orchestra. A local-first kanban board for orchestrating AI coding agents.*

-----

## Philosophy

Maestro is an **orchestration layer**, not an AI product. It doesn’t provide AI — it provides a control surface for local AI agents. The app never holds API keys or manages subscriptions. That’s the agent’s problem.

The user’s workflow has two distinct phases:

1. **Exploration** — Brainstorming, research, planning with AI assistance. Happens before code exists.
1. **Implementation** — AI agents execute tasks against a codebase in isolated workspaces.

Both phases use the same underlying mechanism: launch an agent process, stream its output, capture artifacts.

-----

## Tech Stack

|Layer              |Choice                              |Rationale                                                                       |
|-------------------|------------------------------------|--------------------------------------------------------------------------------|
|UI Framework       |**SvelteKit**                       |Minimal boilerplate, built-in reactivity, small bundle                          |
|UI Components      |**shadcn-svelte**                   |Own the code, accessible primitives (Bits UI), Tailwind-based, keyboard-friendly|
|Desktop Shell      |**Tauri**                           |Lightweight, Rust-backed, native fs/process access                              |
|Storage            |**SQLite**                          |Single file, queryable, handles large conversation histories                    |
|Drag & Drop        |**svelte-dnd-action**               |Mature Svelte DnD library                                                       |
|Agent Communication|**Executor pattern + `maestro` CLI**|Agent-agnostic process management, structured two-way communication             |

The app runs as a local web server (`vite dev`) during development and packages as a Tauri desktop app for distribution. No cloud dependencies. No auth.

-----

## Data Model

### Project

A project is the top-level container. It owns cards, settings, and agent configuration.

```
Project {
  id              TEXT PRIMARY KEY
  name            TEXT
  agent_config    JSON        -- which agent to use, flags, profiles
  base_path       TEXT        -- override for project storage location
  created_at      DATETIME
  updated_at      DATETIME
}
```

### Linked Directories

Projects can reference zero or more external directories. These can be git repos, doc folders, or any path on disk. Not required until a card enters a Started status.

```
LinkedDirectory {
  id              TEXT PRIMARY KEY
  project_id      TEXT REFERENCES Project
  path            TEXT        -- absolute path on disk
  label           TEXT        -- user-friendly name
  is_repo         BOOLEAN     -- whether this is a git repository
  created_at      DATETIME
}
```

### Status Groups & Statuses

Fixed groups mirroring Linear. User-defined statuses within each group.

```
StatusGroup (ENUM - not a table, hardcoded):
  - Backlog
  - Unstarted
  - Started
  - Completed
  - Cancelled
```

```
Status {
  id              TEXT PRIMARY KEY
  project_id      TEXT REFERENCES Project
  group           TEXT        -- one of the fixed StatusGroup values
  name            TEXT        -- user-defined (e.g. "In Review", "Soon")
  sort_order      INTEGER
  is_default      BOOLEAN     -- default status for this group
  created_at      DATETIME
}
```

Default statuses created with every new project:

|Group    |Default Statuses      |
|---------|----------------------|
|Backlog  |Backlog               |
|Unstarted|Unstarted             |
|Started  |In Progress, In Review|
|Completed|Completed             |
|Cancelled|Cancelled             |

### Card

The card is the fundamental unit. It moves through statuses and is the anchor point for AI conversations, agent workspaces, and artifacts.

```
Card {
  id              TEXT PRIMARY KEY
  project_id      TEXT REFERENCES Project
  parent_id       TEXT REFERENCES Card  -- NULL for top-level cards
  status_id       TEXT REFERENCES Status
  title           TEXT
  description     TEXT        -- markdown
  labels          JSON        -- array of label strings
  sort_order      INTEGER     -- position within its status column
  created_at      DATETIME
  updated_at      DATETIME
}
```

### Sub-Cards

A card with a `parent_id` is a sub-card. Sub-cards are full cards — they have their own status, conversations, artifacts, and agent workspaces. They do **not** appear as standalone items on the kanban board.

On the board, the parent card shows a progress summary (e.g. “2/4 completed”). The user can focus into a parent card to see and manage its sub-cards as if each were its own card.

### Open Questions

A soft gate encouraging thorough planning before implementation. Open questions surface uncertainty that should be resolved before moving to a Started status.

```
OpenQuestion {
  id              TEXT PRIMARY KEY
  card_id         TEXT REFERENCES Card
  question        TEXT
  resolution      TEXT        -- NULL until resolved
  source          TEXT        -- "agent" | "user"
  resolved_by     TEXT        -- "agent" | "user" | NULL
  created_at      DATETIME
  resolved_at     DATETIME
}
```

**How they’re created:**

- **Agent-generated** — The agent uses `maestro question "<question>"` to surface open questions. Maestro registers them on the card in real time.
- **User-created** — The user can manually add open questions via the UI.

**How they’re resolved:**

- Either the user or the agent can mark a question resolved. The agent uses `maestro resolve-question --id <id>`.

**Soft gate behavior:**

- When the user drags a card to a Started status with unresolved open questions, Maestro shows a warning with the list of unresolved questions.
- The user can override and proceed anyway.
- The card displays an open questions indicator on the board (e.g. “3 open questions”).

### Conversation

AI chat threads attached to a card. A card can have multiple conversations (e.g. separate exploration threads).

```
Conversation {
  id              TEXT PRIMARY KEY
  card_id         TEXT REFERENCES Card
  agent_type      TEXT        -- "claude-code", "codex", "opencode", etc.
  started_at      DATETIME
  ended_at        DATETIME
}
```

```
ConversationMessage {
  id              TEXT PRIMARY KEY
  conversation_id TEXT REFERENCES Conversation
  role            TEXT        -- "user" | "agent"
  content         TEXT
  timestamp       DATETIME
}
```

### Agent Workspace

Linked agent sessions. Auto-attached when a card transitions to a Started status. Can be manually attached at any time.

```
AgentWorkspace {
  id              TEXT PRIMARY KEY
  card_id         TEXT REFERENCES Card
  agent_type      TEXT
  status          TEXT        -- "running" | "paused" | "reviewing" | "completed" | "failed"
  session_id      TEXT        -- agent's session ID for resume capability
  pid             INTEGER     -- OS process ID for re-attachment
  worktree_path   TEXT        -- path to git worktree (if implementation)
  branch_name     TEXT        -- git branch name (if implementation)
  review_count    INTEGER     -- number of review loop iterations
  attached_at     DATETIME
  completed_at    DATETIME
}
```

### Artifact

Files produced by users or agents, stored on disk. Markdown-first.

```
Artifact {
  id              TEXT PRIMARY KEY
  card_id         TEXT REFERENCES Card
  name            TEXT        -- display name
  type            TEXT        -- "markdown" (extensible later)
  path            TEXT        -- relative path within project artifacts dir
  created_by      TEXT        -- "user" | "agent"
  created_at      DATETIME
  updated_at      DATETIME
}
```

-----

## Directory Structure

```
~/.maestro/                           # global default (user-configurable)
  config.toml                         # global settings, default agent, custom base path
  projects/
    {project_id}/
      db.sqlite                       # all project data
      artifacts/
        {card_id}/
          exploration-notes.md        # agent-generated during exploration
          implementation-plan.md      # planning artifacts
      worktrees/
        {card_id}-{branch}/           # git worktrees for implementation
```

### Key Principle

- `artifacts/{card_id}/` is the **neutral zone** — exploration agents run here, producing markdown files that automatically become card artifacts.
- `worktrees/` is for **implementation** — isolated git branches tied to linked repo directories.
- The project directory is independent of any linked directories.

-----

## Agent Executor Pattern

All agent interaction uses the same pattern regardless of phase (exploration or implementation):

```
Executor {
  agent_type      -- resolved from config hierarchy (project → global)
  binary_path     -- from agent profile (or custom_command)
  flags           -- from agent profile
  model           -- resolved per status group from config hierarchy
  working_dir     -- where the agent runs
  system_prompt   -- merged: config instructions for status group + card context
  env             -- environment variables
}
```

### How It Works

1. **Launch** — Spawn the agent as a child process with configured flags and working directory.
1. **Stream** — Pipe stdout/stderr into the UI as a streaming view on the card.
1. **Interact** — User can send messages to stdin.
1. **Structured feedback** — Agent calls `maestro` CLI commands to communicate status changes, open questions, and artifacts back to the app.
1. **Capture** — Files the agent creates in the working directory can also be detected and registered as artifacts.

### Exploration Mode

- Working dir: `artifacts/{card_id}/`
- System prompt includes the Maestro skill file instructing the agent to use `maestro question` and `maestro add-artifact` commands.
- Linked directories provided as read-only context (if any exist).

### Implementation Mode

- Working dir: `worktrees/{card_id}-{branch}/` (a git worktree of a linked repo)
- System prompt includes task description, relevant artifacts from exploration.
- Full read/write access to the worktree.

### Agent Context Composition

When launching an agent session for a card, Maestro assembles context from multiple sources:

**Always included:**

- Card title + description
- Parent card title + description (if sub-card)
- System prompt for the current mode (exploration vs implementation)

**Included by default (user can toggle off):**

- Card’s own artifacts
- Parent card’s artifacts (if sub-card)

**Not included by default (user can toggle on):**

- Parent conversation history
- Sibling card details

**Never auto-included:**

- Unrelated cards’ data

This keeps agent context focused and token-efficient. The user can override per-card if a specific task needs broader or narrower context.

### Priority Agent Support

|Agent      |Status                                          |
|-----------|------------------------------------------------|
|Claude Code|Primary — richest CLI, best programmatic support|
|Codex      |Supported                                       |
|Opencode   |Supported                                       |
|Others     |Future — add new executors as needed            |

Adding a new agent = writing a new executor config. The app is agent-agnostic by design.

-----

## Maestro CLI — Structured Agent Communication

Instead of parsing text output from agents, Maestro ships a **CLI tool** (`maestro`) that agents (and users) can call to communicate back to the running app. Every agent can run shell commands — this is the universal capability.

The CLI communicates with the Maestro app over a local socket (Unix socket or localhost HTTP).

### Commands

```bash
# Open Questions
maestro question "How should we handle token refresh?"
maestro resolve-question --id q_123

# Artifacts
maestro add-artifact --file plan.md
maestro add-artifact --file notes.md --name "Research Notes"

# Status
maestro set-status in-review
maestro set-status in-progress

# Logging
maestro log "Finished implementing auth module"
maestro log --level warn "Skipped test for ee vs. the base branch.
1. Surface a file tree showing added/modified/deleted files.
1. Present inline diff view for each changed file.
1. Enable a “Create PR” action (creates a PR/MR on the linked repo’s remote).

### The Review Loop

Review is a conversation, not a one-shot gate.

```
In Progress → Agent completes → In Review
                                   ↓
                          User reviews diffs
                          User discusses with agent (chat on the card)
                          User requests changes
                                   ↓
                          In Progress → Agent implements feedback → In Review
                                   ↓
                          User approves → Done (or Create PR → Done)
```

During review, the user can:

- **View diffs** — File-by-file inline diff, similar to a PR review.
- **View file tree** — See the full scope of what the agent touched.
- **Chat with the agent** — Discuss thdge case X"

# Card Context (read-only, agent can query for info)
maestro get-card                     # current card details
maestro get-artifacts                # list artifacts on current card
maestro get-parent                   # parent card details (if sub-card)
```

### Agent Skill File

The CLI is paired with a **skill file** — a markdown document injected into the agent’s system prompt that explains the available commands and when to use them. This is the “contract” between Maestro and the agent.

```markdown
# Maestro Integration

You are running inside Maestro, an AI agent orchestration tool. You have
access to the `maestro` CLI to communicate with the app.

## During Exploration/Planning
- Use `maestro question "<question>"` to surface open questions that need
  human input before implementation can begin.
  - Use `maestro resolve-question --id <id>` when a question has been addressed.
  - Use `maestro add-artifact --file <path>` to attach planning documents.

  ## During Implementation
  - Use `maestro set-status in-review` when you've completed your work.
  - Use `maestro add-artifact --file <path>` to attach any generated docs.
  - Use `maestro log "<message>"` to record progress notes.
  ```

  ### Why Not MCP?

  - **CLI works with every agent** — if it can run a shell command, it can talk to Maestro.
  - **Users can call it too** — useful for scripting, automation, and debugging.
  - **No protocol negotiation** — no version compatibility or handshake issues.
  - **Easy to test** — run the command yourself in a terminal.
  - **Ships as a single binary** alongside the Tauri app.

  -----

  ## Process Lifecycle

  Agent sessions are **independent OS processes**. Maestro spawns them but doesn’t own their lifecycle.

  ### Maestro crashes / unexpected quit

  - Agent processes continue running independently.
  - On restart, Maestro checks for active `AgentWorkspace` records with `status = "running"` and attempts to re-attach using stored session IDs.

  ### User quits Maestro intentionally

  - If any agents are running, prompt: “Keep running agents in background, or stop all?”
  - If kept running, re-attach on next launch.

  ### Agent process crashes

  - Maestro detects the process exit and marks the workspace as `"failed"`.
  - Notify the user and offer to resume the session (using the stored session ID so the agent picks up where it left off).
  - Worktree is preserved for inspection.

  ### Session tracking

  ```
  AgentWorkspace {
    ...
      session_id      TEXT        -- agent's session ID for resume capability
        pid             INTEGER     -- OS process ID for re-attachment
	  ...
	  }
	  ```

	  Maestro stores enough state to reconnect to or resume any agent session across app restarts.

	  -----

	  ### → Unstarted (Leaving Backlog)

	  When a card moves out of Backlog into Unstarted:

	  1. If the project has **no linked directories**, prompt: “This project doesn’t have any directories attached. Would you like to link one now?”
	  - Browse for an existing directory
	  - Create a new directory
	  - Skip for now
	  1. Check for unresolved open questions — show soft gate warning if any exist.

	  ### → Started (In Progress)

	  1. **Repo selection** — If the project has:
	  - **No linked repos** → Prompt: “No repository linked. Initialize a new repo or link an existing one?”
	    - Init new repo (creates dir + `git init`, links to project)
	      - Browse for existing repo
	        - Skip (proceed without git isolation — agent runs in artifacts dir)
		- **One repo** → Use it automatically.
		- **Multiple repos** → Prompt: “Which repository should this card’s workspace use?”
		1. If no agent workspace is attached, auto-create one.
		1. If a repo was selected, create a worktree.
		1. Populate the agent’s context with the card description + any attached artifacts.
		1. The user can override any of this or attach workspaces manually at any stage.

		### → Started (In Review)

		Triggered automatically when the agent signals completion (process exits cleanly). Can also be moved manually by the user. When entering review:

		1. Generate a diff summary of all changes in the worktree implementation, ask questions, request changes. This conversation is stored on the card alongside the workspace.
- **Send back** — Move to In Progress with feedback. The agent’s context includes the review conversation so it knows what to fix.
- **Approve** — Move to Done directly, or create a PR first.

The worktree stays alive through the entire review loop. It’s only cleaned up after the card reaches Completed or Cancelled.

### Backward Transitions

Moving a card to an earlier status group is always allowed, but Maestro handles active resources carefully.

**Any backward move with a running agent:**

- Prompt: “An agent is currently running. Stop it, let it finish in the background, or cancel the move?”
- If stopped, workspace status becomes `"paused"`.
- If left running, it continues and the workspace status stays `"running"` — but the card’s status still moves back.

**Started → Unstarted or Backlog** (deprioritize/rethink):

- Worktree is preserved. No work is destroyed.
- All conversations, artifacts, and workspaces remain on the card.
- If the card later moves back to In Progress, the existing worktree is reused.

**Completed → any earlier status** (reopen):

- Old workspaces are archived and remain visible for reference.
- If the card moves to In Progress again, a **new** workspace is created (fresh branch off the current base).

**Cancelled → any earlier status** (un-cancel):

- Same behavior as reopening from Completed.

**General principles:**

- Artifacts and conversations are **never** deleted on status changes.
- Worktrees are **never** auto-deleted on backward moves.
- Old workspaces are always accessible from the card’s Agent tab for reference.

-----

## UI Overview

### Kanban Board

- Columns grouped by fixed status groups (Backlog → Cancelled, left to right).
- Within each group, user-defined statuses are sub-columns or collapsible sections.
- Cards are draggable between statuses.
- Minimal UI — no team features, no permissions, no dashboards.
- All interactive elements must participate in a **focus management system** from the start (even if keyboard shortcuts come later).

### Card Detail View

- Title, description (markdown editor)
- Status selector
- Labels
- **Conversations tab** — list of AI chat threads, streaming view
- **Open Questions tab** — unresolved and resolved questions, add manually, resolve inline
- **Artifacts tab** — list of attached files, inline markdown preview
- **Agent tab** — active/past workspaces, status, streaming output
- **Review tab** (visible when card is in review status):
  - File tree of changed files
  - Inline diff view per file
  - Review conversation thread with the agent
  - “Send Back” action (returns to In Progress with feedback)
  - “Create PR” action
  - “Approve” action (moves to Done)

-----

## Configuration

Settings follow a **global defaults → project overrides** hierarchy. Project-level settings override global defaults. Where noted, settings can also be overridden per status group.

### Global (`~/.maestro/config.toml`)

```toml
[storage]
base_path = "~/.maestro"

# ──────────────────────────────────────
# Agent Profiles
# ──────────────────────────────────────
# Multiple agents can be configured. The user selects which to use.

[agents.claude-code]
binary = "claude"
flags = ["--dangerously-skip-permissions"]
# custom_command overrides binary + flags entirely
# custom_command = "/path/to/my-script.sh"

[agents.codex]
binary = "codex"
flags = ["--full-auto"]

[agents.opencode]
binary = "opencode"

# ──────────────────────────────────────
# Default Agent Selection
# ──────────────────────────────────────
[defaults]
agent = "claude-code"

# ──────────────────────────────────────
# Per-Status Group Settings (global defaults)
# ──────────────────────────────────────
# Each status group can have its own agent, model, and custom instructions.
# These serve as defaults — projects can override any of these.

[defaults.status.backlog]
agent = "claude-code"
model = "sonnet"
instructions = """
You are in exploration/planning mode. Help the user think through ideas, 
research approaches, and surface open questions. Do not write implementation code.
Use `maestro question` to surface open questions and `maestro add-artifact` 
to save planning documents.
"""

[defaults.status.unstarted]
agent = "claude-code"
model = "sonnet"
instructions = """
You are in planning mode. Help the user break down the task into concrete 
steps, define acceptance criteria, and prepare for implementation.
"""

[defaults.status.started]
agent = "claude-code"
model = "sonnet"
instructions = """
You are implementing a task. Follow the plan and artifacts attached to this card.
Write clean, working code.
"""

[defaults.status.completed]
# Typically no agent interaction, but configurable if needed

[defaults.status.cancelled]
# Typically no agent interaction
```

### Per-Project Overrides

Stored in `db.sqlite` as the project’s `agent_config` JSON field. Same structure as global defaults — any field set here takes precedence.

```json
{
  "agent": "codex",
  "status": {
    "started": {
      "agent": "claude-code",
      "model": "opus",
      "instructions": "Additional project-specific instructions..."
    }
  }
}
```

### Resolution Order

For any given card being launched in a status group:

1. **Project-level status override** (if set)
1. **Project-level default** (if set)
1. **Global status override** (if set)
1. **Global default**

This means a user can, for example, use Sonnet for exploration globally but override to Opus for implementation on a specific project — or use Claude Code everywhere except one project that uses Codex.

-----

## What This Is NOT

- Not a team collaboration tool. Single user, local only.
- Not an AI provider. Bring your own agent.
- Not a full project management suite. No sprints, no time tracking, no reporting.
- Not a code editor. It orchestrates agents that edit code.

-----

## Future Considerations (Not MVP, But Don’t Box Out)

### Keyboard-First Navigation

The UI must be architected with a **focus management system** from the start. Even if keyboard shortcuts aren’t implemented in MVP, the component structure should support:

- A global focus context that tracks which element (card, column, tab, dialog) is active.
- Arrow keys + hjkl (vim-style) navigation between cards and columns.
- Action keys for focused items (Enter to open, `n` to create, `x` to archive, etc.)
- Command palette (Cmd+K / Ctrl+K) for quick actions.

Linear is the gold standard here. Every component should be “focusable” from day one, even if the keybindings come later.

### Alternate Views

The kanban board is the default, but the data model is view-agnostic. Cards, statuses, and status groups should have no coupling to horizontal column layout. Future views:

- **Vertical list view** — Grouped by status, collapsible sections.
- **Table view** — Spreadsheet-like, sortable/filterable.
- **Timeline view** — If we ever add dates/estimates.

The architecture already supports this — views are just different renderings of the same card + status data.

-----

## Open Questions for v2+

- **Card relationships** — Blocking, dependencies (defer unless needed early).
- **Agent marketplace/registry** — Community-shared executor configs.
- **Schema migrations** — Pick a migration tool early (e.g. Drizzle, or manual versioned SQL scripts).
- **Git conventions** — Branch naming (e.g. `maestro/{card-id}-{slug}`), remote targeting for PRs, worktree cleanup policy.
- **Search & filtering** — Text search across cards, label filtering on the board.
