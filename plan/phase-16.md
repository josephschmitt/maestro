# Phase 16: Maestro CLI + IPC

**Prerequisites:** Phase 14 (agent executor)
**Goal:** `maestro` CLI binary for structured agent → app communication over a local Unix socket.

## Steps

1. **Rust workspace setup** — Add `cli/` crate to the Cargo workspace (`members = ["src-tauri", "cli"]`).
2. **CLI with clap** — Parse commands: `question`, `resolve-question`, `add-artifact`, `set-status`, `log`, `get-card`, `get-artifacts`, `get-parent`.
3. **Unix socket server** — Tauri app starts a Unix socket server at `/tmp/maestro-{project_id}.sock` on project open.
4. **IPC protocol** — JSON messages over the socket. Request/response pattern.
5. **CLI client** — Reads `MAESTRO_SOCKET` and `MAESTRO_CARD_ID` env vars, connects to socket, sends JSON, reads response.
6. **Server handler** — Routes incoming messages to DB operations + emits Tauri events to update the frontend in real-time.
7. **Agent skill file** — Markdown document explaining available commands, injected into system prompts.
8. **Env var injection** — When spawning agents, set `MAESTRO_SOCKET` and `MAESTRO_CARD_ID`.

## Key Files to Create

```
Cargo.toml                           — Workspace root (members)
cli/Cargo.toml                       — CLI crate dependencies (clap, serde_json, tokio)
cli/src/main.rs                      — CLI entry point
cli/src/commands/question.rs         — maestro question "<text>"
cli/src/commands/resolve_question.rs — maestro resolve-question --id <id>
cli/src/commands/artifact.rs         — maestro add-artifact --file <path>
cli/src/commands/status.rs           — maestro set-status <status>
cli/src/commands/log.rs              — maestro log "<message>"
cli/src/commands/get_card.rs         — maestro get-card (returns JSON)
cli/src/commands/get_artifacts.rs    — maestro get-artifacts (returns JSON)
cli/src/commands/get_parent.rs       — maestro get-parent (returns JSON)
cli/src/ipc.rs                       — Unix socket client
src-tauri/src/ipc/mod.rs             — Socket server module
src-tauri/src/ipc/server.rs          — Unix socket server lifecycle
src-tauri/src/ipc/handler.rs         — Route messages to handlers
src-tauri/src/ipc/protocol.rs       — Message types (JSON schema)
assets/maestro-skill.md              — Agent skill file template
```

## Key Files to Modify

```
src-tauri/src/main.rs                — Start IPC server on app launch
src-tauri/src/executor/spawn.rs      — Inject MAESTRO_SOCKET + MAESTRO_CARD_ID env vars
src-tauri/src/executor/context.rs    — Include skill file in system prompt
```

## IPC Protocol

```json
// Request (CLI → App)
{
  "command": "question",
  "card_id": "abc-123",
  "payload": { "question": "How should we handle auth?" }
}

// Response (App → CLI)
{
  "ok": true,
  "data": { "id": "q_456" }
}
```

## Skill File (assets/maestro-skill.md)

```markdown
# Maestro Integration

You are running inside Maestro, an AI agent orchestration tool.
You have access to the `maestro` CLI.

## During Exploration/Planning
- `maestro question "<question>"` — surface open questions
- `maestro resolve-question --id <id>` — resolve a question
- `maestro add-artifact --file <path>` — attach documents

## During Implementation
- `maestro set-status in-review` — signal work is complete
- `maestro add-artifact --file <path>` — attach generated docs
- `maestro log "<message>"` — record progress notes
```

## Key Details

- Socket path: `/tmp/maestro-{project_id}.sock`. Cleaned up on app exit.
- Server uses tokio: `UnixListener::bind(path)` → accept connections → read JSON → handle → write response
- Each CLI invocation opens a new connection, sends one request, reads one response, exits
- Handler performs DB operations using the same connection/commands as Tauri, then emits events: `app.emit("question-created", question)`, etc.
- Frontend listens for these events and updates stores in real-time (questions appear as agent creates them)
- CLI binary ships alongside the Tauri app. Path to CLI binary set as env var or placed in a well-known location.
