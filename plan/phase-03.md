# Phase 3: Global Config + Project Management

**Prerequisites:** Phase 2 (SQLite data layer)
**Goal:** Create/open/switch between projects. Read and write `~/.maestro/config.toml`. Seed default statuses on project creation.

## Steps

1. **Global config parsing** — Define `GlobalConfig` Rust struct, parse `~/.maestro/config.toml` with the `toml` crate. Create default config if missing on first launch.
2. **Config resolution logic** — Merge function: project `agent_config` overrides global defaults, per-status-group settings override general defaults.
3. **Project creation flow** — Generate UUID, create directory tree (`~/.maestro/projects/{id}/`, `artifacts/`, `worktrees/`), create `db.sqlite`, run migrations, seed default statuses.
4. **Project listing** — Scan `~/.maestro/projects/` for existing projects, read each `db.sqlite` to get project metadata.
5. **Project switcher UI** — Dropdown in the sidebar showing all projects, current project highlighted.
6. **Create project dialog** — Simple dialog with name input + create button.
7. **Auto-open last project** — Store `last_project_id` in global config, auto-open on launch.
8. **Svelte store** — `project` writable store for current project; all downstream components react to it.

## Key Files to Create

```
src-tauri/src/config/mod.rs          — Config module entry
src-tauri/src/config/global.rs       — TOML parsing, GlobalConfig struct
src-tauri/src/config/resolution.rs   — Merge logic (project overrides global)
src-tauri/src/commands/config.rs     — Tauri commands for config read/write
src/lib/services/config.ts           — Frontend config service
src/lib/stores/project.ts            — Svelte store for current project
src/lib/components/project-switcher.svelte  — Project selector dropdown
src/lib/components/create-project-dialog.svelte — New project dialog
```

## Key Files to Modify

```
src-tauri/src/main.rs                — Register config commands, init config on startup
src/routes/+layout.svelte            — Include project switcher in sidebar
```

## Default Statuses (seeded on project creation)

| Group     | Status Name  | sort_order | is_default |
|-----------|-------------|------------|------------|
| Backlog   | Backlog     | 0          | true       |
| Unstarted | Unstarted   | 0          | true       |
| Started   | In Progress | 0          | true       |
| Started   | In Review   | 1          | false      |
| Completed | Completed   | 0          | true       |
| Cancelled | Cancelled   | 0          | true       |

## Global Config Structure

```toml
[storage]
base_path = "~/.maestro"

[agents.claude-code]
binary = "claude"
flags = ["--dangerously-skip-permissions"]

[agents.codex]
binary = "codex"
flags = ["--full-auto"]

[defaults]
agent = "claude-code"
last_project_id = ""

[defaults.status.backlog]
agent = "claude-code"
model = "sonnet"
instructions = "..."

# ... similar for other status groups
```

## Key Details

- If `~/.maestro/` doesn't exist, create it with a default `config.toml`
- Project creation is atomic: if any step fails (db, directories), roll back
- The project store emits events that cause downstream stores (cards, statuses) to reload
- Config resolution order: project status override → project default → global status override → global default
- `base_path` in config allows users to store Maestro data elsewhere (e.g., a synced folder)
