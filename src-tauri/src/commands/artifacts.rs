use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;
use crate::executor::{EventBus, MaestroEvent};
use crate::fs::artifacts::{
    delete_artifact_file, ensure_artifact_dir, name_to_slug, read_artifact_file,
    write_artifact_file,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub id: String,
    pub card_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub path: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_artifact(row: &rusqlite::Row) -> Result<Artifact, rusqlite::Error> {
    Ok(Artifact {
        id: row.get(0)?,
        card_id: row.get(1)?,
        name: row.get(2)?,
        artifact_type: row.get(3)?,
        path: row.get(4)?,
        created_by: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

const ARTIFACT_SELECT: &str = "\
    SELECT id, card_id, name, type, path, created_by, created_at, updated_at \
    FROM artifacts";

fn resolve_artifact_path(base_path: &Path, project_id: &str, relative_path: &str) -> PathBuf {
    base_path
        .join("projects")
        .join(project_id)
        .join(relative_path)
}

pub fn create_artifact_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
    name: &str,
    content: &str,
    created_by: &str,
) -> Result<Artifact, String> {
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

        if created_by != "agent" && created_by != "user" {
            return Err(format!(
                "Invalid created_by: {created_by}. Must be 'agent' or 'user'"
            ));
        }

        let slug = name_to_slug(name);
        if slug.is_empty() {
            return Err("Artifact name must contain at least one alphanumeric character".to_string());
        }

        let relative_path = format!("artifacts/{card_id}/{slug}.md");

        let existing_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE card_id = ?1 AND path = ?2",
                rusqlite::params![card_id, relative_path],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check for duplicate: {e}"))?;

        let relative_path = if existing_count > 0 {
            let id_suffix = &uuid::Uuid::new_v4().to_string()[..8];
            format!("artifacts/{card_id}/{slug}-{id_suffix}.md")
        } else {
            relative_path
        };

        let base = PathBuf::from(&base_path);
        ensure_artifact_dir(&base, project_id, card_id)?;

        let full_path = resolve_artifact_path(&base_path, project_id, &relative_path);
        write_artifact_file(&full_path, content)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO artifacts (id, card_id, name, type, path, created_by, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'markdown', ?4, ?5, ?6, ?7)",
            rusqlite::params![id, card_id, name, relative_path, created_by, now, now],
        )
        .map_err(|e| format!("Failed to create artifact: {e}"))?;

        conn.query_row(
            &format!("{ARTIFACT_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_artifact,
        )
        .map_err(|e| format!("Failed to read created artifact: {e}"))
    })
}

#[tauri::command]
pub fn create_artifact(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
    project_id: String,
    card_id: String,
    name: String,
    content: String,
    created_by: String,
) -> Result<Artifact, String> {
    let result = create_artifact_inner(&config, &project_id, &card_id, &name, &content, &created_by)?;
    event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(result)
}

pub fn read_artifact_inner(
    config: &ConfigState,
    project_id: &str,
    id: &str,
) -> Result<String, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let relative_path: String = conn
            .query_row(
                "SELECT path FROM artifacts WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Artifact {id} not found: {e}"))?;

        let full_path = resolve_artifact_path(&base_path, project_id, &relative_path);
        read_artifact_file(&full_path)
    })
}

#[tauri::command]
pub fn read_artifact(
    config: State<ConfigState>,
    project_id: String,
    id: String,
) -> Result<String, String> {
    read_artifact_inner(&config, &project_id, &id)
}

pub fn update_artifact_inner(
    config: &ConfigState,
    project_id: &str,
    id: &str,
    content: &str,
) -> Result<Artifact, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let relative_path: String = conn
            .query_row(
                "SELECT path FROM artifacts WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Artifact {id} not found: {e}"))?;

        let full_path = resolve_artifact_path(&base_path, project_id, &relative_path);
        write_artifact_file(&full_path, content)?;

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE artifacts SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )
        .map_err(|e| format!("Failed to update artifact: {e}"))?;

        conn.query_row(
            &format!("{ARTIFACT_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_artifact,
        )
        .map_err(|e| format!("Failed to read updated artifact: {e}"))
    })
}

#[tauri::command]
pub fn update_artifact(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
    project_id: String,
    id: String,
    content: String,
) -> Result<Artifact, String> {
    let result = update_artifact_inner(&config, &project_id, &id, &content)?;
    event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(result)
}

pub fn delete_artifact_inner(
    config: &ConfigState,
    project_id: &str,
    id: &str,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let relative_path: Result<String, _> = conn.query_row(
            "SELECT path FROM artifacts WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        );

        if let Ok(relative_path) = relative_path {
            let full_path = resolve_artifact_path(&base_path, project_id, &relative_path);
            delete_artifact_file(&full_path)?;
        }

        let rows_affected = conn
            .execute(
                "DELETE FROM artifacts WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("Failed to delete artifact: {e}"))?;

        if rows_affected == 0 {
            return Err(format!("Artifact {id} not found"));
        }

        Ok(())
    })
}

#[tauri::command]
pub fn delete_artifact(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
    project_id: String,
    id: String,
) -> Result<(), String> {
    delete_artifact_inner(&config, &project_id, &id)?;
    event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(())
}

pub fn list_artifacts_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
) -> Result<Vec<Artifact>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{ARTIFACT_SELECT} WHERE card_id = ?1 ORDER BY created_at DESC"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![card_id], |row| row_to_artifact(row))
            .map_err(|e| format!("Failed to query artifacts: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read artifact row: {e}"))
    })
}

#[tauri::command]
pub fn list_artifacts(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<Vec<Artifact>, String> {
    list_artifacts_inner(&config, &project_id, &card_id)
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

    fn insert_artifact(conn: &rusqlite::Connection, card_id: &str, name: &str, path: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO artifacts (id, card_id, name, type, path, created_by, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'markdown', ?4, 'user', ?5, ?6)",
            rusqlite::params![id, card_id, name, path, now, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_create_and_list_artifacts() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            insert_artifact(conn, &card_id, "Plan", "artifacts/card/plan.md");
            insert_artifact(conn, &card_id, "Notes", "artifacts/card/notes.md");

            let mut stmt = conn
                .prepare(&format!("{ARTIFACT_SELECT} WHERE card_id = ?1 ORDER BY created_at DESC"))
                .unwrap();
            let artifacts: Vec<Artifact> = stmt
                .query_map(rusqlite::params![card_id], |row| row_to_artifact(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(artifacts.len(), 2);
            assert_eq!(artifacts[0].name, "Notes");
            assert_eq!(artifacts[1].name, "Plan");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_artifact() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let a_id = insert_artifact(conn, &card_id, "To delete", "artifacts/card/delete.md");

            conn.execute("DELETE FROM artifacts WHERE id = ?1", rusqlite::params![a_id])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM artifacts WHERE id = ?1",
                    rusqlite::params![a_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_cascade_delete_on_card_removal() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            insert_artifact(conn, &card_id, "A1", "artifacts/card/a1.md");
            insert_artifact(conn, &card_id, "A2", "artifacts/card/a2.md");

            conn.execute("DELETE FROM cards WHERE id = ?1", rusqlite::params![card_id])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM artifacts WHERE card_id = ?1",
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
