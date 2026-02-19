use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbConnection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub agent_config: serde_json::Value,
    pub base_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn create_project(db: State<DbConnection>, name: String) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let agent_config = serde_json::json!({});

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, name, agent_config.to_string(), now, now],
        )
        .map_err(|e| format!("Failed to create project: {e}"))?;

        Ok(Project {
            id,
            name,
            agent_config,
            base_path: None,
            created_at: now.clone(),
            updated_at: now,
        })
    })
}

#[tauri::command]
pub fn get_project(db: State<DbConnection>, id: String) -> Result<Project, String> {
    db.with_conn(|conn| {
        conn.query_row(
            "SELECT id, name, agent_config, base_path, created_at, updated_at FROM projects WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                let config_str: String = row.get(2)?;
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    agent_config: serde_json::from_str(&config_str).unwrap_or(serde_json::json!({})),
                    base_path: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("Project not found: {e}"))
    })
}

#[tauri::command]
pub fn list_projects(db: State<DbConnection>) -> Result<Vec<Project>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, agent_config, base_path, created_at, updated_at FROM projects ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to query projects: {e}"))?;

        let projects = stmt
            .query_map([], |row| {
                let config_str: String = row.get(2)?;
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    agent_config: serde_json::from_str(&config_str).unwrap_or(serde_json::json!({})),
                    base_path: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to read projects: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect projects: {e}"))?;

        Ok(projects)
    })
}

#[tauri::command]
pub fn update_project(
    db: State<DbConnection>,
    id: String,
    name: Option<String>,
    agent_config: Option<serde_json::Value>,
    base_path: Option<String>,
) -> Result<Project, String> {
    db.with_conn(|conn| {
        let existing = conn
            .query_row(
                "SELECT id, name, agent_config, base_path, created_at FROM projects WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    let config_str: String = row.get(2)?;
                    Ok((
                        row.get::<_, String>(1)?,
                        serde_json::from_str::<serde_json::Value>(&config_str)
                            .unwrap_or(serde_json::json!({})),
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .map_err(|e| format!("Project not found: {e}"))?;

        let new_name = name.unwrap_or(existing.0);
        let new_config = agent_config.unwrap_or(existing.1);
        let new_base_path = base_path.or(existing.2);
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE projects SET name = ?1, agent_config = ?2, base_path = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![new_name, new_config.to_string(), new_base_path, now, id],
        )
        .map_err(|e| format!("Failed to update project: {e}"))?;

        Ok(Project {
            id,
            name: new_name,
            agent_config: new_config,
            base_path: new_base_path,
            created_at: existing.3,
            updated_at: now,
        })
    })
}

#[tauri::command]
pub fn delete_project(db: State<DbConnection>, id: String) -> Result<(), String> {
    db.with_conn(|conn| {
        let changes = conn
            .execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to delete project: {e}"))?;

        if changes == 0 {
            return Err("Project not found".to_string());
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbConnection;

    fn test_db() -> DbConnection {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.sqlite");
        let db = DbConnection::open(&path).unwrap();
        // Keep tempdir alive by leaking it (test only)
        std::mem::forget(dir);
        db
    }

    #[test]
    fn test_create_and_get_project() {
        let db = test_db();
        let project = db
            .with_conn(|conn| {
                let id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, ?2, '{}', ?3, ?4)",
                    rusqlite::params![id, "Test Project", now, now],
                )
                .map_err(|e| format!("{e}"))?;

                conn.query_row(
                    "SELECT id, name, agent_config, base_path, created_at, updated_at FROM projects WHERE id = ?1",
                    rusqlite::params![id],
                    |row| {
                        let config_str: String = row.get(2)?;
                        Ok(Project {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            agent_config: serde_json::from_str(&config_str).unwrap_or(serde_json::json!({})),
                            base_path: row.get(3)?,
                            created_at: row.get(4)?,
                            updated_at: row.get(5)?,
                        })
                    },
                )
                .map_err(|e| format!("{e}"))
            })
            .unwrap();

        assert_eq!(project.name, "Test Project");
        assert!(project.base_path.is_none());
    }

    #[test]
    fn test_list_projects() {
        let db = test_db();
        db.with_conn(|conn| {
            for i in 0..3 {
                let id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, ?2, '{}', ?3, ?4)",
                    rusqlite::params![id, format!("Project {i}"), now, now],
                )
                .map_err(|e| format!("{e}"))?;
            }

            let mut stmt = conn
                .prepare("SELECT COUNT(*) FROM projects")
                .map_err(|e| format!("{e}"))?;
            let count: i64 = stmt.query_row([], |row| row.get(0)).map_err(|e| format!("{e}"))?;
            assert_eq!(count, 3);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_project_cascades() {
        let db = test_db();
        db.with_conn(|conn| {
            let project_id = uuid::Uuid::new_v4().to_string();
            let status_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, ?2, '{}', ?3, ?4)",
                rusqlite::params![project_id, "Test", now, now],
            ).map_err(|e| format!("{e}"))?;

            conn.execute(
                "INSERT INTO statuses (id, project_id, \"group\", name, sort_order, is_default, created_at) VALUES (?1, ?2, 'Backlog', 'Backlog', 0, 1, ?3)",
                rusqlite::params![status_id, project_id, now],
            ).map_err(|e| format!("{e}"))?;

            conn.execute(
                "DELETE FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
            ).map_err(|e| format!("{e}"))?;

            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM statuses WHERE project_id = ?1", rusqlite::params![project_id], |row| row.get(0))
                .map_err(|e| format!("{e}"))?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }
}
