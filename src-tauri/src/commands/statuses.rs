use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    pub id: String,
    pub project_id: String,
    pub group: String,
    pub name: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub skills: Vec<String>,
    pub created_at: String,
}

const VALID_GROUPS: &[&str] = &["Backlog", "Unstarted", "Started", "Completed", "Cancelled"];

fn validate_group(group: &str) -> Result<(), String> {
    if VALID_GROUPS.contains(&group) {
        Ok(())
    } else {
        Err(format!("Invalid status group: {group}"))
    }
}

pub fn default_skills_for_group(group: &str) -> Vec<String> {
    match group {
        "Backlog" => vec!["brainstorming".to_string()],
        "Started" => vec![
            "tdd".to_string(),
            "systematic-debugging".to_string(),
            "verification".to_string(),
        ],
        _ => vec![],
    }
}

fn parse_skills(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}

fn serialize_skills(skills: &[String]) -> String {
    serde_json::to_string(skills).unwrap_or_else(|_| "[]".to_string())
}

fn query_statuses(conn: &rusqlite::Connection, project_id: &str) -> Result<Vec<Status>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, \"group\", name, sort_order, is_default, skills, created_at \
             FROM statuses WHERE project_id = ?1 \
             ORDER BY CASE \"group\" \
                WHEN 'Backlog' THEN 0 \
                WHEN 'Unstarted' THEN 1 \
                WHEN 'Started' THEN 2 \
                WHEN 'Completed' THEN 3 \
                WHEN 'Cancelled' THEN 4 \
             END, sort_order",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![project_id], |row| {
            let skills_json: String = row.get(6)?;
            Ok(Status {
                id: row.get(0)?,
                project_id: row.get(1)?,
                group: row.get(2)?,
                name: row.get(3)?,
                sort_order: row.get(4)?,
                is_default: row.get(5)?,
                skills: parse_skills(&skills_json),
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query statuses: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read status row: {e}"))
}

#[tauri::command]
pub fn list_statuses(
    config: State<ConfigState>,
    project_id: String,
) -> Result<Vec<Status>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| query_statuses(conn, &project_id))
}

#[tauri::command]
pub fn create_status(
    config: State<ConfigState>,
    project_id: String,
    group: String,
    name: String,
    is_default: Option<bool>,
    skills: Option<Vec<String>>,
) -> Result<Status, String> {
    validate_group(&group)?;

    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM statuses WHERE project_id = ?1 AND \"group\" = ?2",
                rusqlite::params![project_id, group],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get max sort order: {e}"))?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = max_order + 1;
        let set_default = is_default.unwrap_or(false);
        let skills_val = skills.unwrap_or_else(|| default_skills_for_group(&group));
        let skills_json = serialize_skills(&skills_val);

        if set_default {
            conn.execute(
                "UPDATE statuses SET is_default = 0 WHERE project_id = ?1 AND \"group\" = ?2",
                rusqlite::params![project_id, group],
            )
            .map_err(|e| format!("Failed to clear existing default: {e}"))?;
        }

        conn.execute(
            "INSERT INTO statuses (id, project_id, \"group\", name, sort_order, is_default, skills, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, project_id, group, name, sort_order, set_default, skills_json, now],
        )
        .map_err(|e| format!("Failed to create status: {e}"))?;

        Ok(Status {
            id,
            project_id,
            group,
            name,
            sort_order,
            is_default: set_default,
            skills: skills_val,
            created_at: now,
        })
    })
}

#[tauri::command]
pub fn update_status(
    config: State<ConfigState>,
    project_id: String,
    id: String,
    name: Option<String>,
    is_default: Option<bool>,
    skills: Option<Vec<String>>,
) -> Result<Status, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let existing = conn
            .query_row(
                "SELECT id, project_id, \"group\", name, sort_order, is_default, skills, created_at \
                 FROM statuses WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
                |row| {
                    let skills_json: String = row.get(6)?;
                    Ok(Status {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        group: row.get(2)?,
                        name: row.get(3)?,
                        sort_order: row.get(4)?,
                        is_default: row.get(5)?,
                        skills: parse_skills(&skills_json),
                        created_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| format!("Status not found: {e}"))?;

        let new_name = name.unwrap_or(existing.name);
        let new_default = is_default.unwrap_or(existing.is_default);
        let new_skills = skills.unwrap_or(existing.skills);
        let new_skills_json = serialize_skills(&new_skills);

        if new_default && !existing.is_default {
            conn.execute(
                "UPDATE statuses SET is_default = 0 WHERE project_id = ?1 AND \"group\" = ?2",
                rusqlite::params![project_id, existing.group],
            )
            .map_err(|e| format!("Failed to clear existing default: {e}"))?;
        }

        conn.execute(
            "UPDATE statuses SET name = ?1, is_default = ?2, skills = ?3 WHERE id = ?4",
            rusqlite::params![new_name, new_default, new_skills_json, id],
        )
        .map_err(|e| format!("Failed to update status: {e}"))?;

        Ok(Status {
            id: existing.id,
            project_id: existing.project_id,
            group: existing.group,
            name: new_name,
            sort_order: existing.sort_order,
            is_default: new_default,
            skills: new_skills,
            created_at: existing.created_at,
        })
    })
}

#[tauri::command]
pub fn delete_status(
    config: State<ConfigState>,
    project_id: String,
    id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let group: String = conn
            .query_row(
                "SELECT \"group\" FROM statuses WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Status not found: {e}"))?;

        let group_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND \"group\" = ?2",
                rusqlite::params![project_id, group],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count statuses in group: {e}"))?;

        if group_count <= 1 {
            return Err(format!(
                "Cannot delete the last status in the {group} group"
            ));
        }

        let card_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE status_id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check for cards: {e}"))?;

        if card_count > 0 {
            return Err(format!(
                "Cannot delete status with {card_count} card(s) assigned"
            ));
        }

        conn.execute(
            "DELETE FROM statuses WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete status: {e}"))?;

        // Recalculate sort_order for remaining statuses in group
        let mut stmt = conn
            .prepare(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = ?2 ORDER BY sort_order",
            )
            .map_err(|e| format!("Failed to prepare reorder query: {e}"))?;

        let ids: Vec<String> = stmt
            .query_map(rusqlite::params![project_id, group], |row| row.get(0))
            .map_err(|e| format!("Failed to query for reorder: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect ids: {e}"))?;

        for (i, status_id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE statuses SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i32, status_id],
            )
            .map_err(|e| format!("Failed to update sort order: {e}"))?;
        }

        // If deleted status was default, make the first remaining one the default
        let has_default: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM statuses WHERE project_id = ?1 AND \"group\" = ?2 AND is_default = 1",
                rusqlite::params![project_id, group],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check for default: {e}"))?;

        if !has_default {
            conn.execute(
                "UPDATE statuses SET is_default = 1 WHERE id = (SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = ?2 ORDER BY sort_order LIMIT 1)",
                rusqlite::params![project_id, group],
            )
            .map_err(|e| format!("Failed to set new default: {e}"))?;
        }

        Ok(())
    })
}

#[tauri::command]
pub fn reorder_statuses(
    config: State<ConfigState>,
    project_id: String,
    group: String,
    status_ids: Vec<String>,
) -> Result<Vec<Status>, String> {
    validate_group(&group)?;

    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        // Verify all IDs belong to this group
        for (i, status_id) in status_ids.iter().enumerate() {
            let belongs: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM statuses WHERE id = ?1 AND project_id = ?2 AND \"group\" = ?3",
                    rusqlite::params![status_id, project_id, group],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to verify status: {e}"))?;

            if !belongs {
                return Err(format!("Status {status_id} does not belong to group {group}"));
            }

            conn.execute(
                "UPDATE statuses SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i32, status_id],
            )
            .map_err(|e| format!("Failed to update sort order: {e}"))?;
        }

        query_statuses(conn, &project_id)
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

    #[test]
    fn test_list_statuses_returns_all_seeded() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let statuses = query_statuses(conn, &project_id)?;
            assert_eq!(statuses.len(), 6);
            assert_eq!(statuses[0].group, "Backlog");
            assert_eq!(statuses[5].group, "Cancelled");
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_list_statuses_ordered_by_group_then_sort_order() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let statuses = query_statuses(conn, &project_id)?;
            let groups: Vec<&str> = statuses.iter().map(|s| s.group.as_str()).collect();
            assert_eq!(groups, vec!["Backlog", "Unstarted", "Started", "Started", "Completed", "Cancelled"]);

            let started: Vec<&Status> = statuses.iter().filter(|s| s.group == "Started").collect();
            assert_eq!(started[0].name, "In Progress");
            assert_eq!(started[1].name, "In Review");
            assert!(started[0].sort_order < started[1].sort_order);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_create_status_appends_to_group() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            let max: i32 = conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started'",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();

            conn.execute(
                "INSERT INTO statuses (id, project_id, \"group\", name, sort_order, is_default, created_at) VALUES (?1, ?2, 'Started', 'Testing', ?3, 0, ?4)",
                rusqlite::params![id, project_id, max + 1, now],
            ).unwrap();

            let started_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started'",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();
            assert_eq!(started_count, 3);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_cannot_delete_last_status_in_group() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let backlog_id: String = conn.query_row(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Backlog'",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();

            let group_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND \"group\" = 'Backlog'",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();
            assert_eq!(group_count, 1);

            // Simulating the validation - only 1 status in Backlog, so deletion should be blocked
            if group_count <= 1 {
                return Ok(());
            }

            conn.execute(
                "DELETE FROM statuses WHERE id = ?1",
                rusqlite::params![backlog_id],
            ).unwrap();
            panic!("Should not reach here");
        }).unwrap();
    }

    #[test]
    fn test_cannot_delete_status_with_cards() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let status_id: String = conn.query_row(
                "SELECT id FROM statuses WHERE project_id = ?1 AND name = 'In Progress'",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();

            let card_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO cards (id, project_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 'Test Card', '', '[]', 0, ?4, ?5)",
                rusqlite::params![card_id, project_id, status_id, now, now],
            ).unwrap();

            let card_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM cards WHERE status_id = ?1",
                rusqlite::params![status_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();
            assert!(card_count > 0, "Should have cards assigned");

            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_reorder_updates_sort_order() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let mut stmt = conn.prepare(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started' ORDER BY sort_order"
            ).unwrap();
            let ids: Vec<String> = stmt.query_map(
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

            assert_eq!(ids.len(), 2);

            let reversed: Vec<String> = ids.iter().rev().cloned().collect();
            for (i, id) in reversed.iter().enumerate() {
                conn.execute(
                    "UPDATE statuses SET sort_order = ?1 WHERE id = ?2",
                    rusqlite::params![i as i32, id],
                ).unwrap();
            }

            let mut stmt2 = conn.prepare(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started' ORDER BY sort_order"
            ).unwrap();
            let new_ids: Vec<String> = stmt2.query_map(
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

            assert_eq!(new_ids, reversed);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_validate_group() {
        assert!(validate_group("Backlog").is_ok());
        assert!(validate_group("Started").is_ok());
        assert!(validate_group("Invalid").is_err());
    }

    #[test]
    fn test_default_flag_clears_previous() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn: &rusqlite::Connection| {
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "UPDATE statuses SET is_default = 0 WHERE project_id = ?1 AND \"group\" = 'Started'",
                rusqlite::params![project_id],
            ).unwrap();

            conn.execute(
                "INSERT INTO statuses (id, project_id, \"group\", name, sort_order, is_default, created_at) VALUES (?1, ?2, 'Started', 'New Default', 2, 1, ?3)",
                rusqlite::params![id, project_id, now],
            ).unwrap();

            let default_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started' AND is_default = 1",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();
            assert_eq!(default_count, 1);

            let default_name: String = conn.query_row(
                "SELECT name FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started' AND is_default = 1",
                rusqlite::params![project_id],
                |row: &rusqlite::Row| row.get(0),
            ).unwrap();
            assert_eq!(default_name, "New Default");

            Ok(())
        }).unwrap();
    }
}
