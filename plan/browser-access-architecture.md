# Maestro — Browser Access Architecture

*Extending Maestro to serve the UI over HTTP so users can connect via any browser, while the Tauri desktop app remains the primary host.*

-----

## Overview

Today, Maestro's frontend runs exclusively inside Tauri's embedded webview. This document describes how to embed an HTTP server in the Tauri app so the same UI is accessible from a standard browser — with full feature parity, real-time synchronization between desktop and browser clients, and optional network exposure for remote access.

### Design Goals

1. **Full parity** — Browser clients can do everything the desktop webview can: CRUD operations, agent launching, output streaming, config changes.
2. **Real-time sync** — Changes made in the desktop webview are reflected in the browser (and vice versa) within milliseconds.
3. **Localhost by default** — The HTTP server binds to `127.0.0.1`. Users can opt in to `0.0.0.0` for network access.
4. **No separate server process** — The HTTP server runs inside the Tauri app. No additional binary to manage.
5. **Minimal frontend changes** — Leverage the existing `tauriInvoke()` abstraction to swap transport without touching components or stores.

-----

## Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| HTTP Framework | **Axum** | Tokio-native (same runtime as Tauri v2), tower middleware ecosystem, excellent WebSocket support, lightweight |
| Real-time Transport | **WebSocket** | Bidirectional (needed for agent stdin), lower overhead than SSE for high-frequency output streaming |
| Serialization | **JSON** (existing serde) | Already used everywhere — Tauri commands, IPC protocol, mock system. No new format. |
| Static Asset Serving | **tower-http `ServeDir`** | Serves SvelteKit build output. Zero-config, supports compression, ETags, range requests. |
| Authentication | **Bearer token** (optional) | Simple shared secret for network-exposed mode. Not needed for localhost. |

### Why Axum Over Alternatives

- **Actix-web**: Runs its own runtime (`actix-rt`). Conflicts with Tauri's tokio runtime. Requires `std::thread::spawn` workaround.
- **Warp**: Good but less actively developed. Axum has stronger ecosystem momentum.
- **Rocket**: Heavier, opinionated. Overkill for an embedded API server.
- **Axum**: Shares Tauri's tokio runtime directly. No thread boundary issues. Tower middleware for CORS, compression, auth.

-----

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    Tauri App Process                   │
│                                                        │
│  ┌────────────┐      ┌─────────────────────────────┐  │
│  │ Tauri       │      │ Axum HTTP Server             │  │
│  │ Webview     │      │                              │  │
│  │ (Desktop)   │      │  GET  /              → UI    │  │
│  │             │      │  POST /api/{command} → API   │  │
│  │  invoke()───┤      │  WS   /ws/events    → Sync  │  │
│  │             │      │  WS   /ws/agent/:id → Agent  │  │
│  └──────┬──────┘      └──────────┬───────────────────┘  │
│         │                        │                       │
│         ▼                        ▼                       │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Shared State Layer                    │   │
│  │                                                    │   │
│  │  ConfigState (Mutex<GlobalConfig>)                │   │
│  │  AgentRegistry (Mutex<HashMap<String, Handle>>)   │   │
│  │  IpcServer (Unix socket manager)                  │   │
│  │  EventBus (broadcast channel) ← NEW              │   │
│  │  SQLite (WAL mode, per-project)                   │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
         │                        │
         ▼                        ▼
    Browser (WebView)        Browser (HTTP)
    localhost:1420           localhost:3100
```

### Key Insight: Shared State, Not Separate Backends

The Tauri commands and Axum handlers share the **same** in-memory state and database connections. There is no duplication of business logic. Axum handlers call the same Rust functions that Tauri commands call — they just receive their arguments from HTTP instead of IPC.

-----

## Component Design

### 1. Axum Server Embedding

The HTTP server starts during Tauri's `setup()` hook, on the same tokio runtime:

```rust
// src-tauri/src/http/server.rs

pub async fn start_http_server(
    config: Arc<ConfigState>,
    registry: Arc<AgentRegistry>,
    ipc_server: Arc<IpcServer>,
    event_bus: Arc<EventBus>,
    bind_addr: SocketAddr,
) -> Result<(), String> {
    let state = AppState { config, registry, ipc_server, event_bus };

    let app = Router::new()
        // Static assets (SvelteKit build output)
        .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
        // API routes
        .nest("/api", api_routes())
        // WebSocket routes
        .route("/ws/events", get(ws_events_handler))
        .route("/ws/agent/:workspace_id", get(ws_agent_handler))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Static asset location**: In production, SvelteKit builds to `dist/` inside the Tauri bundle. The same files served to the webview are served over HTTP. No separate build step.

### 2. API Layer — Command-to-Endpoint Mapping

Rather than designing a RESTful API from scratch, we use an **RPC-style** mapping that mirrors the existing Tauri command surface exactly. This is simpler to implement, easier to maintain (1:1 correspondence), and aligns with how the frontend already works (command name + args object).

```
POST /api/{command_name}
Content-Type: application/json
Body: { ...args }

Response: 200 OK with JSON body, or 400/500 with error string
```

**Examples:**

```
POST /api/list_cards         { "project_id": "abc-123" }
POST /api/create_card        { "project_id": "abc-123", "title": "Fix bug" }
POST /api/launch_agent       { "project_id": "abc-123", "card_id": "def-456", "status_id": "ghi-789" }
POST /api/send_agent_input   { "workspace_id": "jkl-012", "text": "yes" }
```

**Implementation pattern — shared handler functions:**

```rust
// src-tauri/src/http/routes.rs

async fn handle_command(
    State(state): State<AppState>,
    Path(command): Path<String>,
    Json(args): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    match command.as_str() {
        "list_cards" => {
            let project_id = args["project_id"].as_str().ok_or("missing project_id")?;
            let result = crate::commands::cards::list_cards_inner(&state.config, project_id)?;
            Ok(Json(serde_json::to_value(result)?))
        }
        "launch_agent" => {
            // Agent commands need the event_bus for streaming
            let result = crate::commands::agent::launch_agent_inner(
                &state.config, &state.registry, &state.event_bus, args
            ).await?;
            Ok(Json(serde_json::to_value(result)?))
        }
        _ => Err(AppError::NotFound(format!("Unknown command: {command}")))
    }
}
```

**Refactoring pattern**: Each existing Tauri command gets split into:
1. **`_inner()` function** — Pure business logic, takes typed args, returns `Result<T, String>`. Shared by both Tauri and Axum.
2. **Tauri wrapper** — `#[tauri::command]` that extracts state and calls `_inner()`.
3. **Axum wrapper** — Route handler that extracts JSON args and calls `_inner()`.

This is a mechanical refactor. No logic changes.

### 3. Complete Command Catalog

All 55 commands, grouped by module:

**Config (3 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `get_global_config` | — | `GlobalConfigResponse` |
| `set_last_project` | `project_id` | `()` |
| `resolve_config` | `project_agent_config, status_group` | `ResolvedAgentConfigResponse` |

**Projects (5 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `create_project` | `name` | `Project` |
| `get_project` | `id` | `Project` |
| `list_projects` | — | `Vec<ProjectSummary>` |
| `update_project` | `id, name?, agent_config?, base_path?` | `Project` |
| `delete_project` | `id` | `()` |

**Statuses (5 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `list_statuses` | `project_id` | `Vec<Status>` |
| `create_status` | `project_id, group, name, is_default?, status_prompts?` | `Status` |
| `update_status` | `project_id, id, name?, is_default?, status_prompts?` | `Status` |
| `delete_status` | `project_id, id` | `()` |
| `reorder_statuses` | `project_id, group, status_ids` | `Vec<Status>` |

**Cards (8 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `create_card` | `project_id, title, description?, labels?, parent_id?, status_id?` | `CardWithStatus` |
| `get_card` | `project_id, id` | `CardWithStatus` |
| `update_card` | `project_id, id, title?, description?, labels?` | `CardWithStatus` |
| `delete_card` | `project_id, id` | `()` |
| `list_cards` | `project_id` | `Vec<CardWithStatus>` |
| `list_sub_cards` | `project_id, parent_id` | `Vec<CardWithStatus>` |
| `move_card` | `project_id, id, target_status_id, target_sort_order` | `CardWithStatus` |
| `reorder_cards` | `project_id, status_id, card_ids` | `Vec<CardWithStatus>` |

**Questions (6 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `create_question` | `project_id, card_id, question, source` | `OpenQuestion` |
| `list_questions` | `project_id, card_id` | `Vec<OpenQuestion>` |
| `resolve_question` | `project_id, id, resolution?, resolved_by` | `OpenQuestion` |
| `unresolve_question` | `project_id, id` | `OpenQuestion` |
| `delete_question` | `project_id, id` | `()` |
| `count_unresolved_questions` | `project_id, card_ids` | `Vec<(String, i32)>` |

**Artifacts (5 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `create_artifact` | `project_id, card_id, name, content, created_by` | `Artifact` |
| `read_artifact` | `project_id, id` | `String` |
| `update_artifact` | `project_id, id, content` | `Artifact` |
| `delete_artifact` | `project_id, id` | `()` |
| `list_artifacts` | `project_id, card_id` | `Vec<Artifact>` |

**Directories (3 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `add_linked_directory` | `project_id, path, label` | `LinkedDirectory` |
| `remove_linked_directory` | `project_id, id` | `()` |
| `list_linked_directories` | `project_id` | `Vec<LinkedDirectory>` |

**Conversations (5 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `create_conversation` | `project_id, card_id, agent_type` | `Conversation` |
| `list_conversations` | `project_id, card_id` | `Vec<Conversation>` |
| `create_message` | `project_id, conversation_id, role, content` | `ConversationMessage` |
| `list_messages` | `project_id, conversation_id` | `Vec<ConversationMessage>` |
| `count_conversation_messages` | `project_id, conversation_ids` | `Vec<(String, i32)>` |

**Agent (8 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `launch_agent` | `project_id, card_id, status_id, worktree_path?, branch_name?, repo_path?` | `AgentWorkspace` |
| `send_agent_input` | `workspace_id, text` | `()` |
| `stop_agent` | `project_id, workspace_id` | `AgentWorkspace` |
| `resume_agent` | `project_id, workspace_id, card_id` | `AgentWorkspace` |
| `list_workspaces` | `project_id, card_id` | `Vec<AgentWorkspace>` |
| `get_workspace` | `project_id, workspace_id` | `AgentWorkspace` |
| `list_running_workspaces` | — | `Vec<AgentWorkspace>` |
| `stop_all_agents` | — | `()` |

**Worktrees (5 commands)**
| Command | Args | Returns |
|---------|------|---------|
| `generate_branch_name` | `card_id, title` | `String` |
| `create_worktree` | `project_id, card_id, repo_path, branch_name` | `String` |
| `check_worktree_exists` | `project_id, card_id, branch_slug` | `Option<String>` |
| `get_card_worktree` | `project_id, card_id` | `Option<WorktreeInfo>` |
| `get_claude_worktree_path` | `repo_path, card_id, title` | `String` |

**IPC (2 commands — desktop-only, not exposed via HTTP)**
| Command | Args | Returns |
|---------|------|---------|
| `start_ipc_server` | `project_id` | `String` |
| `stop_ipc_server` | `project_id` | `()` |

**Total: 55 commands. 53 exposed via HTTP. 2 desktop-only (IPC server management).**

### 4. EventBus — Real-Time Synchronization

The core mechanism for multi-client sync. A new shared component that replaces direct `app.emit()` calls.

```rust
// src-tauri/src/events/bus.rs

use tokio::sync::broadcast;

pub struct EventBus {
    tx: broadcast::Sender<MaestroEvent>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MaestroEvent {
    pub event_type: String,        // e.g., "agent-output", "card-updated", "question-created"
    pub scope: Option<String>,     // e.g., workspace_id for agent events
    pub payload: serde_json::Value,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub fn emit(&self, event: MaestroEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MaestroEvent> {
        self.tx.subscribe()
    }
}
```

**Dual emission**: When any state-changing operation occurs (via Tauri command OR HTTP API), it emits to:
1. **Tauri events** (`app.emit()`) — for the desktop webview
2. **EventBus** (`event_bus.emit()`) — for all WebSocket clients

This ensures both the desktop and browser clients see the same updates.

**Event categories:**

| Category | Events | Frequency |
|----------|--------|-----------|
| **Agent output** | `agent-output-{id}` | High (per-line, real-time) |
| **Agent lifecycle** | `agent-exit-{id}`, `agent-crashed` | Low |
| **Data mutations** | `card-updated`, `card-moved`, `status-changed`, etc. | Medium |
| **IPC events** | `question-created`, `artifact-added`, `card-status-changed` | Medium |

### 5. WebSocket Endpoints

Two WebSocket endpoints serve different purposes:

#### `/ws/events` — Global Event Stream

Streams all `MaestroEvent` messages to connected clients. Used for:
- Data mutation notifications (card created, status changed, etc.)
- Agent lifecycle events (crash, exit)
- IPC-originated events (agent asked question, added artifact)

```rust
async fn ws_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let mut rx = state.event_bus.subscribe();
        let (mut sender, mut receiver) = socket.split();

        // Forward events to client
        let send_task = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let msg = Message::Text(serde_json::to_string(&event).unwrap());
                if sender.send(msg).await.is_err() { break; }
            }
        });

        // Handle client messages (ping/pong, close)
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Close(_) = msg { break; }
            }
        });

        tokio::select! { _ = send_task => {}, _ = recv_task => {} }
    })
}
```

#### `/ws/agent/:workspace_id` — Agent I/O Stream

Bidirectional WebSocket for a specific agent session:
- **Server → Client**: stdout/stderr lines (same as `agent-output-{id}` Tauri events)
- **Client → Server**: stdin input (same as `send_agent_input` command)

```rust
async fn ws_agent_handler(
    ws: WebSocketUpgrade,
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let mut rx = state.event_bus.subscribe();
        let (mut sender, mut receiver) = socket.split();

        // Forward agent output to client (filtered by workspace_id)
        let wid = workspace_id.clone();
        let send_task = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if event.scope.as_deref() == Some(&wid)
                   && event.event_type.starts_with("agent-") {
                    let msg = Message::Text(serde_json::to_string(&event).unwrap());
                    if sender.send(msg).await.is_err() { break; }
                }
            }
        });

        // Forward client input to agent stdin
        let registry = state.registry.clone();
        let wid2 = workspace_id.clone();
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    if let Some(tx) = registry.get_stdin_tx(&wid2) {
                        let _ = tx.send(text).await;
                    }
                }
            }
        });

        tokio::select! { _ = send_task => {}, _ = recv_task => {} }
    })
}
```

### 6. Frontend Transport Abstraction

The existing `tauriInvoke()` function in `src/lib/services/db.ts` is the single point of change:

```typescript
// src/lib/services/db.ts

const HTTP_BASE = import.meta.env.VITE_MAESTRO_HTTP_URL; // e.g., "http://localhost:3100"

function getTransportMode(): 'tauri' | 'http' | 'mock' {
    if (isTauriAvailable()) return 'tauri';
    if (HTTP_BASE) return 'http';
    return 'mock';
}

export async function tauriInvoke<T>(
    command: string,
    args?: Record<string, unknown>
): Promise<T> {
    const mode = getTransportMode();

    switch (mode) {
        case 'tauri':
            return invoke<T>(command, args);

        case 'http': {
            const response = await fetch(`${HTTP_BASE}/api/${command}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(args ?? {}),
            });
            if (!response.ok) {
                throw new Error(await response.text());
            }
            return response.json();
        }

        case 'mock': {
            const { dispatchMockCommand } = await import('./mock/index.js');
            return dispatchMockCommand<T>(command, args);
        }
    }
}
```

**Event listening** also needs a transport-aware abstraction:

```typescript
// src/lib/services/events.ts

let globalWs: WebSocket | null = null;
const listeners = new Map<string, Set<(payload: unknown) => void>>();

function ensureWebSocket() {
    if (globalWs) return;
    globalWs = new WebSocket(`${HTTP_BASE.replace('http', 'ws')}/ws/events`);
    globalWs.onmessage = (msg) => {
        const event = JSON.parse(msg.data);
        const callbacks = listeners.get(event.event_type);
        callbacks?.forEach(cb => cb(event.payload));
    };
    globalWs.onclose = () => {
        globalWs = null;
        // Reconnect after delay
        setTimeout(ensureWebSocket, 1000);
    };
}

export async function listenEvent<T>(
    eventName: string,
    callback: (payload: T) => void
): Promise<() => void> {
    const mode = getTransportMode();

    if (mode === 'tauri') {
        const { listen } = await import('@tauri-apps/api/event');
        return listen<T>(eventName, (e) => callback(e.payload));
    }

    if (mode === 'http') {
        ensureWebSocket();
        if (!listeners.has(eventName)) listeners.set(eventName, new Set());
        listeners.get(eventName)!.add(callback as (payload: unknown) => void);
        return () => listeners.get(eventName)?.delete(callback as (payload: unknown) => void);
    }

    // Mock mode: no-op
    return () => {};
}
```

### 7. Multi-Client Synchronization Strategy

**Problem**: When Client A creates a card, Client B's store is stale until it reloads.

**Solution**: Event-driven invalidation with targeted refetching.

#### How it works:

1. **Mutation commands emit change events.** Every state-changing `_inner()` function emits a `MaestroEvent` describing what changed.

2. **Clients subscribe to change events.** Via Tauri events (desktop) or WebSocket (browser).

3. **Stores react to change events by refetching.** The existing `loadCards()`, `loadStatuses()`, etc. functions are already designed for full reloads. We add event listeners that trigger these reloads.

```typescript
// src/lib/stores/cards.ts (additions)

import { listenEvent } from '$lib/services/events';

// Auto-reload when any card changes
listenEvent('cards-changed', (payload: { project_id: string }) => {
    const project = get(currentProject);
    if (project?.id === payload.project_id) {
        loadCards();  // Already exists, triggers full reload
    }
});
```

#### Why full reload instead of incremental patches:

1. **Simpler** — No conflict resolution, no OT, no CRDT. Just refetch.
2. **Already works** — Every mutation already calls `loadCards()` locally. We're just making remote mutations do the same.
3. **Correct** — SQLite is the single source of truth. No stale cache.
4. **Fast enough** — Loading all cards for a project is a single indexed query. Sub-millisecond for typical board sizes (< 1000 cards).

#### Mutation event catalog:

| Event | Emitted by | Triggers reload of |
|-------|------------|-------------------|
| `cards-changed` | create, update, delete, move, reorder card | `loadCards()` |
| `statuses-changed` | create, update, delete, reorder status | `loadStatuses()` |
| `questions-changed` | create, resolve, unresolve, delete question | `loadQuestions()` |
| `artifacts-changed` | create, update, delete artifact | `loadArtifacts()` |
| `conversations-changed` | create conversation, create message | `loadConversations()` |
| `workspaces-changed` | launch, stop, resume agent | `loadWorkspaces()` |
| `projects-changed` | create, update, delete project | `loadProjects()` |
| `directories-changed` | add, remove linked directory | `loadDirectories()` |
| `config-changed` | update global config | `loadConfig()` |

-----

## Configuration

### Server Settings

Added to `~/.maestro/config.toml`:

```toml
[http_server]
enabled = true                    # Start HTTP server on app launch
port = 3100                       # Port to bind
bind_address = "127.0.0.1"       # Localhost only (default)
# bind_address = "0.0.0.0"       # Expose to network (user opt-in)
auth_token = ""                   # Optional bearer token for network mode
```

### Frontend Detection

The SvelteKit app detects its environment at runtime:

- **Tauri webview**: `__TAURI_INTERNALS__` exists → use `invoke()`
- **Browser with HTTP_BASE**: `VITE_MAESTRO_HTTP_URL` set → use `fetch()` + WebSocket
- **Browser without HTTP_BASE**: Fall back to mock (development mode)

The HTTP server injects `VITE_MAESTRO_HTTP_URL` into the served HTML via a `<script>` tag:

```html
<script>window.__MAESTRO_HTTP_URL__ = "http://localhost:3100";</script>
```

-----

## Security

### Localhost Mode (Default)

- Server binds to `127.0.0.1` — only accessible from the same machine.
- No authentication required (same trust model as Tauri webview).
- CORS allows `localhost` origins.

### Network Mode (Opt-In)

When `bind_address = "0.0.0.0"`:

- **Auth token required** — All HTTP and WebSocket requests must include `Authorization: Bearer <token>`.
- **Token generation** — App generates a random token on first network-mode startup, stored in config.
- **CORS restricted** — Only configured origins allowed.
- **Warning on startup** — App shows a prominent warning that the server is network-accessible.

### What's NOT in Scope

- HTTPS/TLS — Users who want encrypted remote access should use a reverse proxy (nginx, Caddy) or SSH tunnel. Embedding TLS adds certificate management complexity.
- User accounts / multi-user — This is still a single-user app. Multiple browser tabs share the same session.

-----

## File Structure

New files to create:

```
src-tauri/src/
  http/
    mod.rs              # Module declaration
    server.rs           # Axum server setup, static serving, middleware
    routes.rs           # API route definitions + command dispatch
    websocket.rs        # WebSocket handlers (events + agent I/O)
    auth.rs             # Bearer token middleware (optional)
  events/
    mod.rs              # Module declaration
    bus.rs              # EventBus (broadcast channel)
```

Files to modify:

```
src-tauri/src/
  lib.rs                # Start HTTP server in setup(), manage EventBus
  commands/*.rs         # Extract _inner() functions from each command
  executor/stream.rs    # Emit to EventBus in addition to Tauri events
  executor/lifecycle.rs # Emit to EventBus in addition to Tauri events
  executor/monitor.rs   # Emit to EventBus in addition to Tauri events
  ipc/handler.rs        # Emit to EventBus in addition to Tauri events

src-tauri/Cargo.toml    # Add axum, tower-http, tokio-tungstenite

src/lib/services/
  db.ts                 # Add HTTP transport mode to tauriInvoke()
  events.ts             # NEW: Transport-aware event listening abstraction

src/lib/stores/
  *.ts                  # Add event listeners for cross-client sync
```

-----

## Implementation Phases

### Phase 1: Foundation (EventBus + Server Skeleton)
- Create `EventBus` with tokio broadcast channel
- Embed Axum server in Tauri setup
- Serve static assets
- Add config settings for port/bind address
- Health check endpoint (`GET /api/health`)

### Phase 2: API Layer (Command Extraction + HTTP Routes)
- Refactor all 53 commands to extract `_inner()` functions
- Create Axum route handler with command dispatch
- Wire up shared state (ConfigState, AgentRegistry)
- Add error mapping (Rust errors → HTTP status codes)

### Phase 3: Real-Time (WebSocket + Event Emission)
- Implement `/ws/events` global event stream
- Implement `/ws/agent/:workspace_id` per-agent stream
- Add EventBus emission to all mutation paths (commands, executor, IPC handler)
- Dual emission: Tauri events + EventBus

### Phase 4: Frontend Transport (HTTP + WebSocket Client)
- Modify `tauriInvoke()` to support HTTP mode
- Create `listenEvent()` abstraction over Tauri events and WebSocket
- Add WebSocket reconnection logic
- Update stores to listen for cross-client change events

### Phase 5: Security + Polish
- Bearer token authentication for network mode
- CORS configuration
- Settings UI for HTTP server (enable/disable, port, bind address)
- Connection status indicator in browser UI
- Startup warning for network mode

-----

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Two clients move same card simultaneously | Card ends up in unexpected position | Last-write-wins (SQLite serializes writes). Immediate refetch resolves UI inconsistency. |
| WebSocket disconnects | Browser client misses events, stale UI | Auto-reconnect + full state reload on reconnect |
| High-frequency agent output overwhelms WebSocket | Browser lag, dropped messages | Buffer + batch output lines (send every 50ms instead of per-line) |
| Tauri tokio runtime conflicts with Axum | Server fails to start | Both use tokio natively — no conflict expected. Axum was chosen specifically for this. |
| Network-exposed server without auth | Unauthorized access to agent control | Auth token required when bind != localhost. Config validation enforces this. |
| Static asset path differs in dev vs production | 404s in browser | Use Tauri's resource path resolver for production, fallback to `dist/` for dev |

-----

## What This Does NOT Change

- **Tauri webview behavior** — Desktop app works exactly as before. The HTTP server is additive.
- **IPC system** — Unix socket for CLI communication is unchanged. The `maestro` CLI still talks to the IPC server, not the HTTP server.
- **Database schema** — No new tables or columns. EventBus is in-memory only.
- **Build pipeline** — SvelteKit still builds to static. No SSR. No new build targets.
- **Agent spawning** — Agents run as local processes regardless of which client triggered the launch.
