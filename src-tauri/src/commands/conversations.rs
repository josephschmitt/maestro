use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub card_id: String,
    pub agent_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

fn row_to_conversation(row: &rusqlite::Row) -> Result<Conversation, rusqlite::Error> {
    Ok(Conversation {
        id: row.get(0)?,
        card_id: row.get(1)?,
        agent_type: row.get(2)?,
        started_at: row.get(3)?,
        ended_at: row.get(4)?,
    })
}

fn row_to_message(row: &rusqlite::Row) -> Result<ConversationMessage, rusqlite::Error> {
    Ok(ConversationMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        timestamp: row.get(4)?,
    })
}

const CONVERSATION_SELECT: &str = "\
    SELECT id, card_id, agent_type, started_at, ended_at \
    FROM conversations";

const MESSAGE_SELECT: &str = "\
    SELECT id, conversation_id, role, content, timestamp \
    FROM conversation_messages";

pub fn create_conversation_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
    agent_type: &str,
) -> Result<Conversation, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let card_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![card_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify card: {e}"))?;

        if !card_exists {
            return Err(format!("Card {card_id} not found in project"));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO conversations (id, card_id, agent_type, started_at) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, card_id, agent_type, now],
        )
        .map_err(|e| format!("Failed to create conversation: {e}"))?;

        conn.query_row(
            &format!("{CONVERSATION_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_conversation,
        )
        .map_err(|e| format!("Failed to read created conversation: {e}"))
    })
}

#[tauri::command]
pub fn create_conversation(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    agent_type: String,
) -> Result<Conversation, String> {
    create_conversation_inner(&config, &project_id, &card_id, &agent_type)
}

pub fn list_conversations_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
) -> Result<Vec<Conversation>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{CONVERSATION_SELECT} WHERE card_id = ?1 ORDER BY started_at DESC"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![card_id], |row| row_to_conversation(row))
            .map_err(|e| format!("Failed to query conversations: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read conversation row: {e}"))
    })
}

#[tauri::command]
pub fn list_conversations(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<Vec<Conversation>, String> {
    list_conversations_inner(&config, &project_id, &card_id)
}

pub fn create_message_inner(
    config: &ConfigState,
    project_id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
) -> Result<ConversationMessage, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        if role != "user" && role != "agent" {
            return Err(format!("Invalid role: {role}. Must be 'user' or 'agent'"));
        }

        let conv_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM conversations WHERE id = ?1",
                rusqlite::params![conversation_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify conversation: {e}"))?;

        if !conv_exists {
            return Err(format!("Conversation {conversation_id} not found"));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO conversation_messages (id, conversation_id, role, content, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, conversation_id, role, content, now],
        )
        .map_err(|e| format!("Failed to create message: {e}"))?;

        conn.query_row(
            &format!("{MESSAGE_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_message,
        )
        .map_err(|e| format!("Failed to read created message: {e}"))
    })
}

#[tauri::command]
pub fn create_message(
    config: State<ConfigState>,
    project_id: String,
    conversation_id: String,
    role: String,
    content: String,
) -> Result<ConversationMessage, String> {
    create_message_inner(&config, &project_id, &conversation_id, &role, &content)
}

pub fn list_messages_inner(
    config: &ConfigState,
    project_id: &str,
    conversation_id: &str,
) -> Result<Vec<ConversationMessage>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{MESSAGE_SELECT} WHERE conversation_id = ?1 ORDER BY timestamp"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![conversation_id], |row| row_to_message(row))
            .map_err(|e| format!("Failed to query messages: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read message row: {e}"))
    })
}

#[tauri::command]
pub fn list_messages(
    config: State<ConfigState>,
    project_id: String,
    conversation_id: String,
) -> Result<Vec<ConversationMessage>, String> {
    list_messages_inner(&config, &project_id, &conversation_id)
}

pub fn count_conversation_messages_inner(
    config: &ConfigState,
    project_id: &str,
    conversation_ids: &[String],
) -> Result<Vec<(String, i32)>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let mut results = Vec::new();
        for conv_id in conversation_ids {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM conversation_messages WHERE conversation_id = ?1",
                    rusqlite::params![conv_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to count messages: {e}"))?;
            results.push((conv_id.clone(), count));
        }
        Ok(results)
    })
}

#[tauri::command]
pub fn count_conversation_messages(
    config: State<ConfigState>,
    project_id: String,
    conversation_ids: Vec<String>,
) -> Result<Vec<(String, i32)>, String> {
    count_conversation_messages_inner(&config, &project_id, &conversation_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::projects::seed_default_statuses;
    use crate::db::DbConnection;

    fn setup_test_db() -> (DbConnection, String) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("db.sqlite");
        let db = DbConnection::open(&db_path).unwrap();
        let project_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        db.with_conn(|conn: &rusqlite::Connection| {
            conn.execute(
                "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, 'Test', '{}', ?2, ?3)",
                rusqlite::params![project_id, now, now],
            ).map_err(|e| format!("{e}"))?;
            seed_default_statuses(conn, &project_id)?;
            Ok(())
        }).unwrap();

        (db, project_id)
    }

    fn insert_card(conn: &rusqlite::Connection, project_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status_id: String = conn
            .query_row(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Backlog' AND is_default = 1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO cards (id, project_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'Test Card', '', '[]', 0, ?4, ?5)",
            rusqlite::params![id, project_id, status_id, now, now],
        )
        .unwrap();
        id
    }

    fn insert_conversation(conn: &rusqlite::Connection, card_id: &str, agent_type: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO conversations (id, card_id, agent_type, started_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, card_id, agent_type, now],
        )
        .unwrap();
        id
    }

    fn insert_message(conn: &rusqlite::Connection, conversation_id: &str, role: &str, content: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO conversation_messages (id, conversation_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, conversation_id, role, content, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_create_and_list_conversations() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let c1_id = insert_conversation(conn, &card_id, "claude-code");
            let c2_id = insert_conversation(conn, &card_id, "manual");

            let mut stmt = conn
                .prepare(&format!(
                    "{CONVERSATION_SELECT} WHERE card_id = ?1 ORDER BY started_at DESC"
                ))
                .unwrap();
            let convos: Vec<Conversation> = stmt
                .query_map(rusqlite::params![card_id], |row| row_to_conversation(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(convos.len(), 2);
            assert_eq!(convos[1].id, c1_id);
            assert_eq!(convos[1].agent_type, "claude-code");
            assert_eq!(convos[0].id, c2_id);
            assert_eq!(convos[0].agent_type, "manual");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_create_and_list_messages() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let conv_id = insert_conversation(conn, &card_id, "manual");
            let m1_id = insert_message(conn, &conv_id, "user", "Hello");
            let m2_id = insert_message(conn, &conv_id, "agent", "Hi there");

            let mut stmt = conn
                .prepare(&format!(
                    "{MESSAGE_SELECT} WHERE conversation_id = ?1 ORDER BY timestamp"
                ))
                .unwrap();
            let msgs: Vec<ConversationMessage> = stmt
                .query_map(rusqlite::params![conv_id], |row| row_to_message(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(msgs.len(), 2);
            assert_eq!(msgs[0].id, m1_id);
            assert_eq!(msgs[0].role, "user");
            assert_eq!(msgs[0].content, "Hello");
            assert_eq!(msgs[1].id, m2_id);
            assert_eq!(msgs[1].role, "agent");
            assert_eq!(msgs[1].content, "Hi there");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_cascade_delete_conversation_on_card_removal() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let conv_id = insert_conversation(conn, &card_id, "manual");
            insert_message(conn, &conv_id, "user", "Hello");

            conn.execute("DELETE FROM cards WHERE id = ?1", rusqlite::params![card_id])
                .unwrap();

            let conv_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM conversations WHERE card_id = ?1",
                    rusqlite::params![card_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(conv_count, 0);

            let msg_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM conversation_messages WHERE conversation_id = ?1",
                    rusqlite::params![conv_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(msg_count, 0);
            Ok(())
        })
        .unwrap();
    }
}
