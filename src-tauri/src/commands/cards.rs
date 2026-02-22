use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardWithStatus {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub status_id: String,
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    pub status_name: String,
    pub status_group: String,
}

fn row_to_card(row: &rusqlite::Row) -> Result<CardWithStatus, rusqlite::Error> {
    let labels_json: String = row.get(6)?;
    let labels: Vec<String> =
        serde_json::from_str(&labels_json).unwrap_or_default();
    Ok(CardWithStatus {
        id: row.get(0)?,
        project_id: row.get(1)?,
        parent_id: row.get(2)?,
        status_id: row.get(3)?,
        title: row.get(4)?,
        description: row.get(5)?,
        labels,
        sort_order: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        status_name: row.get(10)?,
        status_group: row.get(11)?,
    })
}

const CARD_SELECT: &str = "\
    SELECT c.id, c.project_id, c.parent_id, c.status_id, c.title, c.description, \
           c.labels, c.sort_order, c.created_at, c.updated_at, \
           s.name AS status_name, s.\"group\" AS status_group \
    FROM cards c JOIN statuses s ON c.status_id = s.id";

#[tauri::command]
pub fn create_card(
    config: State<Arc<ConfigState>>,
    project_id: String,
    title: String,
    description: Option<String>,
    labels: Option<Vec<String>>,
    parent_id: Option<String>,
    status_id: Option<String>,
) -> Result<CardWithStatus, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let resolved_status_id = match status_id {
            Some(sid) => {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM statuses WHERE id = ?1 AND project_id = ?2",
                        rusqlite::params![sid, project_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| format!("Failed to verify status: {e}"))?;
                if !exists {
                    return Err(format!("Status {sid} not found in project"));
                }
                sid
            }
            None => {
                conn.query_row(
                    "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Backlog' AND is_default = 1 LIMIT 1",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .map_err(|_| "No default Backlog status found for project".to_string())?
            }
        };

        if let Some(ref pid) = parent_id {
            let parent_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND project_id = ?2",
                    rusqlite::params![pid, project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to verify parent card: {e}"))?;
            if !parent_exists {
                return Err(format!("Parent card {pid} not found"));
            }
        }

        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM cards WHERE status_id = ?1 AND COALESCE(parent_id, '') = COALESCE(?2, '')",
                rusqlite::params![resolved_status_id, parent_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get max sort order: {e}"))?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let desc = description.unwrap_or_default();
        let labels_vec = labels.unwrap_or_default();
        let labels_json = serde_json::to_string(&labels_vec)
            .map_err(|e| format!("Failed to serialize labels: {e}"))?;
        let sort_order = max_order + 1;

        conn.execute(
            "INSERT INTO cards (id, project_id, parent_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![id, project_id, parent_id, resolved_status_id, title, desc, labels_json, sort_order, now, now],
        )
        .map_err(|e| format!("Failed to create card: {e}"))?;

        let card = conn
            .query_row(
                &format!("{CARD_SELECT} WHERE c.id = ?1"),
                rusqlite::params![id],
                row_to_card,
            )
            .map_err(|e| format!("Failed to read created card: {e}"))?;

        Ok(card)
    })
}

#[tauri::command]
pub fn get_card(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
) -> Result<CardWithStatus, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        conn.query_row(
            &format!("{CARD_SELECT} WHERE c.id = ?1 AND c.project_id = ?2"),
            rusqlite::params![id, project_id],
            row_to_card,
        )
        .map_err(|e| format!("Card not found: {e}"))
    })
}

#[tauri::command]
pub fn update_card(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
    title: Option<String>,
    description: Option<String>,
    labels: Option<Vec<String>>,
) -> Result<CardWithStatus, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let existing = conn
            .query_row(
                "SELECT title, description, labels FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|e| format!("Card not found: {e}"))?;

        let new_title = title.unwrap_or(existing.0);
        let new_description = description.unwrap_or(existing.1);
        let new_labels_json = match labels {
            Some(l) => serde_json::to_string(&l)
                .map_err(|e| format!("Failed to serialize labels: {e}"))?,
            None => existing.2,
        };
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE cards SET title = ?1, description = ?2, labels = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![new_title, new_description, new_labels_json, now, id],
        )
        .map_err(|e| format!("Failed to update card: {e}"))?;

        conn.query_row(
            &format!("{CARD_SELECT} WHERE c.id = ?1"),
            rusqlite::params![id],
            row_to_card,
        )
        .map_err(|e| format!("Failed to read updated card: {e}"))
    })
}

#[tauri::command]
pub fn delete_card(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check card: {e}"))?;

        if !exists {
            return Err("Card not found".to_string());
        }

        conn.execute(
            "DELETE FROM cards WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete card: {e}"))?;

        Ok(())
    })
}

#[tauri::command]
pub fn list_cards(
    config: State<Arc<ConfigState>>,
    project_id: String,
) -> Result<Vec<CardWithStatus>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{CARD_SELECT} WHERE c.project_id = ?1 AND c.parent_id IS NULL ORDER BY c.sort_order"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![project_id], |row| row_to_card(row))
            .map_err(|e| format!("Failed to query cards: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read card row: {e}"))
    })
}

#[tauri::command]
pub fn list_sub_cards(
    config: State<Arc<ConfigState>>,
    project_id: String,
    parent_id: String,
) -> Result<Vec<CardWithStatus>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{CARD_SELECT} WHERE c.parent_id = ?1 AND c.project_id = ?2 ORDER BY c.sort_order"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![parent_id, project_id], |row| {
                row_to_card(row)
            })
            .map_err(|e| format!("Failed to query sub-cards: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read sub-card row: {e}"))
    })
}

#[tauri::command]
pub fn move_card(
    config: State<Arc<ConfigState>>,
    project_id: String,
    id: String,
    target_status_id: String,
    target_sort_order: i32,
) -> Result<CardWithStatus, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let (old_status_id, old_sort_order, parent_id): (String, i32, Option<String>) = conn
            .query_row(
                "SELECT status_id, sort_order, parent_id FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Card not found: {e}"))?;

        let status_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM statuses WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![target_status_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify target status: {e}"))?;

        if !status_exists {
            return Err(format!("Target status {target_status_id} not found"));
        }

        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("Failed to begin transaction: {e}"))?;

        let result = (|| -> Result<CardWithStatus, String> {
            let now = chrono::Utc::now().to_rfc3339();

            // Close the gap in the old status
            if parent_id.is_some() {
                conn.execute(
                    "UPDATE cards SET sort_order = sort_order - 1 WHERE status_id = ?1 AND parent_id = ?2 AND sort_order > ?3",
                    rusqlite::params![old_status_id, parent_id, old_sort_order],
                )
                .map_err(|e| format!("Failed to close gap in old status: {e}"))?;
            } else {
                conn.execute(
                    "UPDATE cards SET sort_order = sort_order - 1 WHERE status_id = ?1 AND parent_id IS NULL AND sort_order > ?2",
                    rusqlite::params![old_status_id, old_sort_order],
                )
                .map_err(|e| format!("Failed to close gap in old status: {e}"))?;
            }

            // Make room in the target status
            if parent_id.is_some() {
                conn.execute(
                    "UPDATE cards SET sort_order = sort_order + 1 WHERE status_id = ?1 AND parent_id = ?2 AND sort_order >= ?3 AND id != ?4",
                    rusqlite::params![target_status_id, parent_id, target_sort_order, id],
                )
                .map_err(|e| format!("Failed to make room in target status: {e}"))?;
            } else {
                conn.execute(
                    "UPDATE cards SET sort_order = sort_order + 1 WHERE status_id = ?1 AND parent_id IS NULL AND sort_order >= ?2 AND id != ?3",
                    rusqlite::params![target_status_id, target_sort_order, id],
                )
                .map_err(|e| format!("Failed to make room in target status: {e}"))?;
            }

            // Move the card
            conn.execute(
                "UPDATE cards SET status_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![target_status_id, target_sort_order, now, id],
            )
            .map_err(|e| format!("Failed to move card: {e}"))?;

            conn.query_row(
                &format!("{CARD_SELECT} WHERE c.id = ?1"),
                rusqlite::params![id],
                row_to_card,
            )
            .map_err(|e| format!("Failed to read moved card: {e}"))
        })();

        match result {
            Ok(card) => {
                conn.execute_batch("COMMIT")
                    .map_err(|e| format!("Failed to commit transaction: {e}"))?;
                Ok(card)
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    })
}

#[tauri::command]
pub fn reorder_cards(
    config: State<Arc<ConfigState>>,
    project_id: String,
    status_id: String,
    card_ids: Vec<String>,
) -> Result<Vec<CardWithStatus>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        // Validate all cards belong to the status before starting transaction
        for card_id in card_ids.iter() {
            let belongs: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND status_id = ?2 AND project_id = ?3",
                    rusqlite::params![card_id, status_id, project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to verify card: {e}"))?;

            if !belongs {
                return Err(format!(
                    "Card {card_id} does not belong to status {status_id}"
                ));
            }
        }

        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("Failed to begin transaction: {e}"))?;

        let result = (|| -> Result<Vec<CardWithStatus>, String> {
            for (i, card_id) in card_ids.iter().enumerate() {
                conn.execute(
                    "UPDATE cards SET sort_order = ?1 WHERE id = ?2",
                    rusqlite::params![i as i32, card_id],
                )
                .map_err(|e| format!("Failed to update sort order: {e}"))?;
            }

            let mut stmt = conn
                .prepare(&format!(
                    "{CARD_SELECT} WHERE c.status_id = ?1 AND c.project_id = ?2 ORDER BY c.sort_order"
                ))
                .map_err(|e| format!("Failed to prepare query: {e}"))?;

            let rows = stmt
                .query_map(rusqlite::params![status_id, project_id], |row| {
                    row_to_card(row)
                })
                .map_err(|e| format!("Failed to query cards: {e}"))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to read card row: {e}"))
        })();

        match result {
            Ok(cards) => {
                conn.execute_batch("COMMIT")
                    .map_err(|e| format!("Failed to commit transaction: {e}"))?;
                Ok(cards)
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
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

    fn get_backlog_status_id(conn: &rusqlite::Connection, project_id: &str) -> String {
        conn.query_row(
            "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Backlog' AND is_default = 1",
            rusqlite::params![project_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn get_status_id_by_name(conn: &rusqlite::Connection, project_id: &str, name: &str) -> String {
        conn.query_row(
            "SELECT id FROM statuses WHERE project_id = ?1 AND name = ?2",
            rusqlite::params![project_id, name],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn insert_card(
        conn: &rusqlite::Connection,
        project_id: &str,
        status_id: &str,
        title: &str,
        sort_order: i32,
        parent_id: Option<&str>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO cards (id, project_id, parent_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, '', '[]', ?6, ?7, ?8)",
            rusqlite::params![id, project_id, parent_id, status_id, title, sort_order, now, now],
        )
        .unwrap();
        id
    }

    fn query_card(conn: &rusqlite::Connection, id: &str) -> CardWithStatus {
        conn.query_row(
            &format!("{CARD_SELECT} WHERE c.id = ?1"),
            rusqlite::params![id],
            row_to_card,
        )
        .unwrap()
    }

    #[test]
    fn test_create_card_defaults_to_backlog() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let card_id = insert_card(conn, &project_id, &backlog_id, "Test Card", 0, None);
            let card = query_card(conn, &card_id);
            assert_eq!(card.status_group, "Backlog");
            assert_eq!(card.title, "Test Card");
            assert!(card.parent_id.is_none());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_create_card_with_labels() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let labels = serde_json::to_string(&vec!["bug", "urgent"]).unwrap();
            conn.execute(
                "INSERT INTO cards (id, project_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 'Bug Card', '', ?4, 0, ?5, ?6)",
                rusqlite::params![id, project_id, backlog_id, labels, now, now],
            )
            .map_err(|e| format!("{e}"))?;

            let card = query_card(conn, &id);
            assert_eq!(card.labels, vec!["bug", "urgent"]);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_list_cards_excludes_sub_cards() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let parent = insert_card(conn, &project_id, &backlog_id, "Parent", 0, None);
            insert_card(conn, &project_id, &backlog_id, "Child", 0, Some(&parent));
            insert_card(conn, &project_id, &backlog_id, "Top Level", 1, None);

            let mut stmt = conn
                .prepare(&format!(
                    "{CARD_SELECT} WHERE c.project_id = ?1 AND c.parent_id IS NULL ORDER BY c.sort_order"
                ))
                .unwrap();
            let cards: Vec<CardWithStatus> = stmt
                .query_map(rusqlite::params![project_id], |row| row_to_card(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(cards.len(), 2);
            assert_eq!(cards[0].title, "Parent");
            assert_eq!(cards[1].title, "Top Level");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_list_sub_cards() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let parent = insert_card(conn, &project_id, &backlog_id, "Parent", 0, None);
            insert_card(conn, &project_id, &backlog_id, "Child A", 0, Some(&parent));
            insert_card(conn, &project_id, &backlog_id, "Child B", 1, Some(&parent));

            let mut stmt = conn
                .prepare(&format!(
                    "{CARD_SELECT} WHERE c.parent_id = ?1 ORDER BY c.sort_order"
                ))
                .unwrap();
            let children: Vec<CardWithStatus> = stmt
                .query_map(rusqlite::params![parent], |row| row_to_card(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(children.len(), 2);
            assert_eq!(children[0].title, "Child A");
            assert_eq!(children[1].title, "Child B");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_move_card_between_statuses() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let in_progress_id = get_status_id_by_name(conn, &project_id, "In Progress");

            let card_a = insert_card(conn, &project_id, &backlog_id, "Card A", 0, None);
            let card_b = insert_card(conn, &project_id, &backlog_id, "Card B", 1, None);

            // Move Card A to In Progress at position 0
            let now = chrono::Utc::now().to_rfc3339();
            // Close gap in backlog
            conn.execute(
                "UPDATE cards SET sort_order = sort_order - 1 WHERE status_id = ?1 AND parent_id IS NULL AND sort_order > 0",
                rusqlite::params![backlog_id],
            ).unwrap();
            // Move card
            conn.execute(
                "UPDATE cards SET status_id = ?1, sort_order = 0, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![in_progress_id, now, card_a],
            ).unwrap();

            let moved = query_card(conn, &card_a);
            assert_eq!(moved.status_id, in_progress_id);
            assert_eq!(moved.sort_order, 0);
            assert_eq!(moved.status_group, "Started");

            // Card B should have moved down
            let card_b_after = query_card(conn, &card_b);
            assert_eq!(card_b_after.sort_order, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_reorder_cards() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let a = insert_card(conn, &project_id, &backlog_id, "A", 0, None);
            let b = insert_card(conn, &project_id, &backlog_id, "B", 1, None);
            let c = insert_card(conn, &project_id, &backlog_id, "C", 2, None);

            // Reorder to C, A, B
            let new_order = vec![&c, &a, &b];
            for (i, card_id) in new_order.iter().enumerate() {
                conn.execute(
                    "UPDATE cards SET sort_order = ?1 WHERE id = ?2",
                    rusqlite::params![i as i32, card_id],
                )
                .unwrap();
            }

            let card_c = query_card(conn, &c);
            let card_a = query_card(conn, &a);
            let card_b = query_card(conn, &b);
            assert_eq!(card_c.sort_order, 0);
            assert_eq!(card_a.sort_order, 1);
            assert_eq!(card_b.sort_order, 2);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_card_cascades_to_sub_cards() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let parent = insert_card(conn, &project_id, &backlog_id, "Parent", 0, None);
            let child = insert_card(conn, &project_id, &backlog_id, "Child", 0, Some(&parent));

            conn.execute("DELETE FROM cards WHERE id = ?1", rusqlite::params![parent])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM cards WHERE id = ?1",
                    rusqlite::params![child],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0, "Sub-card should be cascade deleted");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_update_card_fields() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let card_id = insert_card(conn, &project_id, &backlog_id, "Original", 0, None);

            let new_labels = serde_json::to_string(&vec!["feature"]).unwrap();
            conn.execute(
                "UPDATE cards SET title = 'Updated', description = 'New desc', labels = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![new_labels, chrono::Utc::now().to_rfc3339(), card_id],
            ).unwrap();

            let card = query_card(conn, &card_id);
            assert_eq!(card.title, "Updated");
            assert_eq!(card.description, "New desc");
            assert_eq!(card.labels, vec!["feature"]);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_card_sort_order_appends() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let backlog_id = get_backlog_status_id(conn, &project_id);
            let a = insert_card(conn, &project_id, &backlog_id, "A", 0, None);
            let b = insert_card(conn, &project_id, &backlog_id, "B", 1, None);
            let c = insert_card(conn, &project_id, &backlog_id, "C", 2, None);

            assert_eq!(query_card(conn, &a).sort_order, 0);
            assert_eq!(query_card(conn, &b).sort_order, 1);
            assert_eq!(query_card(conn, &c).sort_order, 2);
            Ok(())
        })
        .unwrap();
    }
}
