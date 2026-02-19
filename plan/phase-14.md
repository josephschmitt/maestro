# Phase 14: Agent Executor — Process Spawning

**Prerequisites:** Phase 12 (linked directories), Phase 13 (conversations)
**Goal:** Spawn agent processes from Tauri, stream output to the UI, accept stdin input. Core execution engine.

## Steps

1. **Agent executor module** — Rust module that resolves agent config, spawns a child process with piped stdio.
2. **Config resolution** — Determine binary, flags, model, instructions from project config → global config hierarchy.
3. **Context assembly** — Build system prompt from: config instructions + card title + card description + skill file.
4. **Process spawning** — Use `std::process::Command` with `Stdio::piped()` for stdin/stdout/stderr.
5. **Output streaming** — Tokio background task reads stdout/stderr line-by-line, emits Tauri events.
6. **AgentWorkspace creation** — Create DB record on agent launch with PID, status="running".
7. **Process lifecycle** — Detect process exit, update workspace status to "completed" (exit 0) or "failed" (non-zero).
8. **Agent tab UI** — Start agent button, streaming terminal output view, input field for sending to stdin.
9. **Exploration mode** — Agent runs in `artifacts/{card_id}/` directory with exploration-mode instructions.

## Key Files to Create

```
src-tauri/src/executor/mod.rs        — Executor module entry
src-tauri/src/executor/spawn.rs      — Process spawning logic
src-tauri/src/executor/stream.rs     — Stdout/stderr streaming via Tauri events
src-tauri/src/executor/lifecycle.rs  — Process monitoring, exit detection
src-tauri/src/executor/context.rs    — Context assembly (prompts, card data)
src-tauri/src/commands/agent.rs      — Tauri commands: launch_agent, send_input, stop_agent
src/lib/services/agent.ts            — Frontend agent service
src/lib/stores/agent.ts              — Agent workspace store, output buffer
src/lib/components/card-detail/tabs/agent-tab.svelte     — Agent tab content
src/lib/components/card-detail/agent-terminal.svelte     — Streaming output display
src/lib/components/card-detail/agent-controls.svelte     — Start/stop/send controls
```

## Key Details

- **Spawning:** `Command::new(binary).args(flags).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).current_dir(working_dir).envs(env_vars).spawn()`
- **Streaming:** Background tokio task: `BufReader::new(stdout).lines()` → for each line, emit `app.emit("agent-output-{workspace_id}", line)`
- **Frontend listening:** `listen('agent-output-{workspace_id}', (event) => outputBuffer.update(b => [...b, event.payload]))`
- **Terminal component:** Monospace font (`font-mono`), auto-scroll to bottom, preserve whitespace. Consider basic ANSI color support (use a lightweight parser).
- **Input:** Text field at bottom of terminal. Enter sends line to agent's stdin via Tauri command `send_input(workspace_id, text)`.
- **Stop:** `stop_agent(workspace_id)` sends SIGTERM to the process, then SIGKILL after 5s timeout.
- **Config resolution for agent:** walk the hierarchy: project status override → project default → global status override → global default. Resolve `binary`, `flags`, `model`.
- **Exploration mode:** working dir = `artifacts/{card_id}/`, system prompt includes exploration instructions from config.
- **Environment variables:** `MAESTRO_CARD_ID={card_id}` always set. `MAESTRO_SOCKET` set if IPC is available (Phase 16).
