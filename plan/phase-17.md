# Phase 17: Process Re-attachment + Crash Recovery

**Prerequisites:** Phase 14 (agent executor)
**Goal:** Handle app restart with agents still running. Detect agent crashes.

## Steps

1. **Startup scan** — On app launch, query AgentWorkspace records with `status = "running"`.
2. **PID validation** — Check if each process is still alive (`kill(pid, 0)` on Unix).
3. **Re-attach if alive** — For agents that support session files (e.g., Claude Code), tail the session log. For raw processes, mark as "running (detached)" — output not available but process continues.
4. **Mark dead as failed** — If PID is gone, update workspace status to "failed". Show notification with resume option.
5. **Background PID monitor** — Tokio interval task (every 5s) checking all "running" workspace PIDs. On death, update status + notify frontend.
6. **Quit dialog** — When user quits with running agents: "Agents are running. Stop all / Keep running in background / Cancel"
7. **Resume capability** — "Resume" button on failed workspaces re-launches agent with stored `session_id` so agents that support it can pick up where they left off.

## Key Files to Create

```
src-tauri/src/executor/reattach.rs   — Re-attachment logic (startup scan)
src-tauri/src/executor/monitor.rs    — Background PID monitor
src/lib/components/dialogs/quit-dialog.svelte          — Quit confirmation
src/lib/components/dialogs/agent-crashed-dialog.svelte — Crash notification + resume
```

## Key Files to Modify

```
src-tauri/src/main.rs                — Run re-attachment scan on startup
src-tauri/src/executor/lifecycle.rs  — Integrate with background monitor
src/lib/components/card-detail/tabs/agent-tab.svelte — Show failed/detached status, resume button
```

## Key Details

- PID check (Unix): `unsafe { libc::kill(pid as i32, 0) }` — returns 0 if process exists
- Re-attachment limitation: cannot re-pipe stdout from an already-running process. Two strategies:
  1. Agents with session files (Claude Code): tail the session file for output
  2. Raw processes: show "Agent is running but output is not available. PID: {pid}"
- Monitor: `tokio::time::interval(Duration::from_secs(5))` → for each running workspace, check PID → if dead, update DB + `app.emit("agent-crashed", workspace_id)`
- Quit handler: Tauri `on_window_event(|event| { if let WindowEvent::CloseRequested { .. } = event { ... } })` — check for running workspaces, show dialog if any
- "Stop all": SIGTERM all running PIDs, wait 5s, SIGKILL survivors, then quit
- "Keep running": just quit — processes continue. Next launch will re-attach.
- Resume: `launch_agent` with `session_id` from the failed workspace. For Claude Code, this means passing `--resume {session_id}`.
