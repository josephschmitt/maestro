use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenQuestion {
    pub id: String,
    pub card_id: String,
    pub question: String,
    pub resolution: Option<String>,
    pub source: String,
    pub resolved_by: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

fn row_to_question(row: &rusqlite::Row) -> Result<OpenQuestion, rusqlite::Error> {
    Ok(OpenQuestion {
        id: row.get(0)?,
        card_id: row.get(1)?,
        question: row.get(2)?,
        resolution: row.get(3)?,
        source: row.get(4)?,
        resolved_by: row.get(5)?,
        created_at: row.get(6)?,
        resolved_at: row.get(7)?,
    })
}

const QUESTION_SELECT: &str = "\
    SELECT id, card_id, question, resolution, source, resolved_by, created_at, resolved_at \
    FROM open_questions";

#[tauri::command]
pub fn create_question(
    config: State<Arc<ConfigState>>,
    project_id: String,
    card_id: String,
    question: String,
    source: String,
) -> Result<OpenQuestion, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

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

        if source != "agent" && source != "user" {
            return Err(format!("Invalid source: {source}. Must be 'agent' or 'user'"));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO open_questions (id, card_id, question, source, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, card_id, question, source, now],
        )
        .map_err(|e| format!("Failed to create question: {e}"))?;

        conn.query_row(
            &format!("{QUESTION_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_question,
        )
        .map_err(|e| format!("Failed to read created question: {e}"))
    })
}

#[tauri::command]
pub fn list_questions(
    config: State<Arc<ConfigState>>,
    project_id: String,
    card_id: String,
) -> Result<Vec<OpenQuestion>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{QUESTION_SELECT} WHERE card_id = ?1 ORDER BY created_at"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![card_id], |row| row_to_question(row))
            .map_err(|e| format!("Failed to query questions: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read question row: {e}"))
    })
}

#[tauri::command]
pub fn resolve_question(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
    resolution: Option<String>,
    resolved_by: String,
) -> Result<OpenQuestion, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        if resolved_by != "agent" && resolved_by != "user" {
            return Err(format!("Invalid resolved_by: {resolved_by}. Must be 'agent' or 'user'"));
        }

        let now = chrono::Utc::now().to_rfc3339();

        let rows_affected = conn
            .execute(
                "UPDATE open_questions SET resolution = ?1, resolved_by = ?2, resolved_at = ?3 WHERE id = ?4",
                rusqlite::params![resolution, resolved_by, now, id],
            )
            .map_err(|e| format!("Failed to resolve question: {e}"))?;

        if rows_affected == 0 {
            return Err(format!("Question {id} not found"));
        }

        conn.query_row(
            &format!("{QUESTION_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_question,
        )
        .map_err(|e| format!("Failed to read resolved question: {e}"))
    })
}

#[tauri::command]
pub fn unresolve_question(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
) -> Result<OpenQuestion, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let rows_affected = conn
            .execute(
                "UPDATE open_questions SET resolution = NULL, resolved_by = NULL, resolved_at = NULL WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("Failed to unresolve question: {e}"))?;

        if rows_affected == 0 {
            return Err(format!("Question {id} not found"));
        }

        conn.query_row(
            &format!("{QUESTION_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_question,
        )
        .map_err(|e| format!("Failed to read unresolved question: {e}"))
    })
}

#[tauri::command]
pub fn delete_question(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let rows_affected = conn
            .execute(
                "DELETE FROM open_questions WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("Failed to delete question: {e}"))?;

        if rows_affected == 0 {
            return Err(format!("Question {id} not found"));
        }

        Ok(())
    })
}

#[tauri::command]
pub fn count_unresolved_questions(
    config: State<Arc<ConfigState>>,
    project_id: String,
    card_ids: Vec<String>,
) -> Result<Vec<(String, i32)>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let mut results = Vec::new();
        for card_id in &card_ids {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM open_questions WHERE card_id = ?1 AND resolved_at IS NULL",
                    rusqlite::params![card_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to count questions: {e}"))?;
            results.push((card_id.clone(), count));
        }
        Ok(results)
    })
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

    fn insert_question(conn: &rusqlite::Connection, card_id: &str, question: &str, source: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO open_questions (id, card_id, question, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, card_id, question, source, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_create_and_list_questions() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let q1_id = insert_question(conn, &card_id, "What API to use?", "user");
            let q2_id = insert_question(conn, &card_id, "Which DB?", "agent");

            let mut stmt = conn
                .prepare(&format!("{QUESTION_SELECT} WHERE card_id = ?1 ORDER BY created_at"))
                .unwrap();
            let questions: Vec<OpenQuestion> = stmt
                .query_map(rusqlite::params![card_id], |row| row_to_question(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(questions.len(), 2);
            assert_eq!(questions[0].id, q1_id);
            assert_eq!(questions[0].source, "user");
            assert!(questions[0].resolved_at.is_none());
            assert_eq!(questions[1].id, q2_id);
            assert_eq!(questions[1].source, "agent");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_resolve_and_unresolve() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let q_id = insert_question(conn, &card_id, "Open question", "user");

            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE open_questions SET resolution = ?1, resolved_by = ?2, resolved_at = ?3 WHERE id = ?4",
                rusqlite::params!["Answered", "user", now, q_id],
            )
            .unwrap();

            let q: OpenQuestion = conn
                .query_row(
                    &format!("{QUESTION_SELECT} WHERE id = ?1"),
                    rusqlite::params![q_id],
                    row_to_question,
                )
                .unwrap();
            assert_eq!(q.resolution, Some("Answered".to_string()));
            assert_eq!(q.resolved_by, Some("user".to_string()));
            assert!(q.resolved_at.is_some());

            conn.execute(
                "UPDATE open_questions SET resolution = NULL, resolved_by = NULL, resolved_at = NULL WHERE id = ?1",
                rusqlite::params![q_id],
            )
            .unwrap();

            let q2: OpenQuestion = conn
                .query_row(
                    &format!("{QUESTION_SELECT} WHERE id = ?1"),
                    rusqlite::params![q_id],
                    row_to_question,
                )
                .unwrap();
            assert!(q2.resolution.is_none());
            assert!(q2.resolved_by.is_none());
            assert!(q2.resolved_at.is_none());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_question() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let q_id = insert_question(conn, &card_id, "To delete", "user");

            conn.execute("DELETE FROM open_questions WHERE id = ?1", rusqlite::params![q_id])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM open_questions WHERE id = ?1",
                    rusqlite::params![q_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_count_unresolved() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            insert_question(conn, &card_id, "Q1", "user");
            insert_question(conn, &card_id, "Q2", "agent");
            let q3_id = insert_question(conn, &card_id, "Q3 resolved", "user");

            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE open_questions SET resolution = 'Done', resolved_by = 'user', resolved_at = ?1 WHERE id = ?2",
                rusqlite::params![now, q3_id],
            )
            .unwrap();

            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM open_questions WHERE card_id = ?1 AND resolved_at IS NULL",
                    rusqlite::params![card_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 2);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_cascade_delete_on_card_removal() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            insert_question(conn, &card_id, "Q1", "user");
            insert_question(conn, &card_id, "Q2", "agent");

            conn.execute("DELETE FROM cards WHERE id = ?1", rusqlite::params![card_id])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM open_questions WHERE card_id = ?1",
                    rusqlite::params![card_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }
}
