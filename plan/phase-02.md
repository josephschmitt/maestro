# Phase 2: SQLite Data Layer

**Prerequisites:** Phase 1 (project scaffold)
**Goal:** Full database schema, migration system, Rust Tauri commands for Project CRUD, and TypeScript types for all entities.

## Steps

1. **Add `rusqlite`** with `bundled` feature to `src-tauri/Cargo.toml`. Also add `uuid` and `serde`/`serde_json`.
2. **Write the full DDL** (`schema.sql`) — all tables from the architecture doc.
3. **Build migration system** — `_migrations` tracking table, versioned SQL scripts, runner that applies unapplied migrations in order.
4. **Create connection manager** — Opens/creates a SQLite database at a given path, runs migrations, returns a connection.
5. **Implement Project CRUD** as Tauri commands — `create_project`, `get_project`, `list_projects`, `update_project`, `delete_project`.
6. **Define TypeScript types** mirroring every database entity.
7. **Create frontend service layer** — Thin wrappers around `invoke()` calls with proper typing.

## Key Files to Create

```
src-tauri/src/db/mod.rs              — Module entry
src-tauri/src/db/schema.sql          — Full DDL (all tables, indexes, foreign keys)
src-tauri/src/db/migrations.rs       — Migration runner
src-tauri/src/db/connection.rs       — Connection pool/management per project
src-tauri/src/commands/mod.rs        — Command registration
src-tauri/src/commands/projects.rs   — Project CRUD Tauri commands
src/lib/types/index.ts               — All TypeScript interfaces
src/lib/types/status.ts              — StatusGroup enum and helpers
src/lib/services/db.ts               — Base invoke wrapper with error handling
src/lib/services/projects.ts         — Project service methods
```

## Schema (all tables)

```sql
-- Project
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  agent_config TEXT DEFAULT '{}',    -- JSON
  base_path TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- LinkedDirectory
CREATE TABLE linked_directories (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  label TEXT NOT NULL,
  is_repo INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

-- Status
CREATE TABLE statuses (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  "group" TEXT NOT NULL,             -- Backlog|Unstarted|Started|Completed|Cancelled
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

-- Card
CREATE TABLE cards (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES cards(id) ON DELETE CASCADE,
  status_id TEXT NOT NULL REFERENCES statuses(id),
  title TEXT NOT NULL,
  description TEXT DEFAULT '',
  labels TEXT DEFAULT '[]',          -- JSON array
  sort_order INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- OpenQuestion
CREATE TABLE open_questions (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  question TEXT NOT NULL,
  resolution TEXT,
  source TEXT NOT NULL,              -- 'agent' | 'user'
  resolved_by TEXT,                  -- 'agent' | 'user' | NULL
  created_at TEXT NOT NULL,
  resolved_at TEXT
);

-- Conversation
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  agent_type TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT
);

-- ConversationMessage
CREATE TABLE conversation_messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  role TEXT NOT NULL,                -- 'user' | 'agent'
  content TEXT NOT NULL,
  timestamp TEXT NOT NULL
);

-- AgentWorkspace
CREATE TABLE agent_workspaces (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  agent_type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'running',
  session_id TEXT,
  pid INTEGER,
  worktree_path TEXT,
  branch_name TEXT,
  review_count INTEGER NOT NULL DEFAULT 0,
  attached_at TEXT NOT NULL,
  completed_at TEXT
);

-- Artifact
CREATE TABLE artifacts (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL DEFAULT 'markdown',
  path TEXT NOT NULL,
  created_by TEXT NOT NULL,          -- 'user' | 'agent'
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

## TypeScript Types

```typescript
type StatusGroup = 'Backlog' | 'Unstarted' | 'Started' | 'Completed' | 'Cancelled';

interface Project {
  id: string;
  name: string;
  agent_config: Record<string, unknown>;
  base_path: string | null;
  created_at: string;
  updated_at: string;
}

// ... similar for all entities
```

## Key Details

- UUIDs generated via Rust `uuid::Uuid::new_v4()` for all IDs
- All timestamps are ISO-8601 strings (`chrono` crate or manual formatting)
- `agent_config` and `labels` are JSON strings in SQLite, parsed to objects in Rust/TS
- Migration system: version 1 = initial schema. Future phases add migrations as needed.
- Tauri commands are `#[tauri::command]` async functions accepting/returning serde structs
- Frontend service methods are typed: `createProject(name: string): Promise<Project>`
- SQLite `PRAGMA foreign_keys = ON` must be set on every connection
