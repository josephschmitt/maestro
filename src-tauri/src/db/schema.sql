-- Maestro Schema v1
-- All IDs are UUIDs stored as TEXT
-- All timestamps are ISO-8601 strings

CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  agent_config TEXT NOT NULL DEFAULT '{}',
  base_path TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE linked_directories (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  label TEXT NOT NULL,
  is_repo INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_linked_directories_project ON linked_directories(project_id);

CREATE TABLE statuses (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  "group" TEXT NOT NULL CHECK("group" IN ('Backlog', 'Unstarted', 'Started', 'Completed', 'Cancelled')),
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_statuses_project ON statuses(project_id);

CREATE TABLE cards (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES cards(id) ON DELETE CASCADE,
  status_id TEXT NOT NULL REFERENCES statuses(id),
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  labels TEXT NOT NULL DEFAULT '[]',
  sort_order INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_cards_project ON cards(project_id);
CREATE INDEX idx_cards_status ON cards(status_id);
CREATE INDEX idx_cards_parent ON cards(parent_id);

CREATE TABLE open_questions (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  question TEXT NOT NULL,
  resolution TEXT,
  source TEXT NOT NULL CHECK(source IN ('agent', 'user')),
  resolved_by TEXT CHECK(resolved_by IN ('agent', 'user')),
  created_at TEXT NOT NULL,
  resolved_at TEXT
);

CREATE INDEX idx_open_questions_card ON open_questions(card_id);

CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  agent_type TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT
);

CREATE INDEX idx_conversations_card ON conversations(card_id);

CREATE TABLE conversation_messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK(role IN ('user', 'agent')),
  content TEXT NOT NULL,
  timestamp TEXT NOT NULL
);

CREATE INDEX idx_conversation_messages_conversation ON conversation_messages(conversation_id);

CREATE TABLE agent_workspaces (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  agent_type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'paused', 'reviewing', 'completed', 'failed')),
  session_id TEXT,
  pid INTEGER,
  worktree_path TEXT,
  branch_name TEXT,
  review_count INTEGER NOT NULL DEFAULT 0,
  attached_at TEXT NOT NULL,
  completed_at TEXT
);

CREATE INDEX idx_agent_workspaces_card ON agent_workspaces(card_id);

CREATE TABLE artifacts (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL DEFAULT 'markdown',
  path TEXT NOT NULL,
  created_by TEXT NOT NULL CHECK(created_by IN ('user', 'agent')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_artifacts_card ON artifacts(card_id);
