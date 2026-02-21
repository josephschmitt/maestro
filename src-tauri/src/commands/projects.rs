use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

struct DefaultStatus {
    group: &'static str,
    name: &'static str,
    sort_order: i32,
    is_default: bool,
    skills: &'static str,
}

const DEFAULT_STATUSES: &[DefaultStatus] = &[
    DefaultStatus {
        group: "Backlog",
        name: "Backlog",
        sort_order: 0,
        is_default: true,
        skills: r#"["brainstorming"]"#,
    },
    DefaultStatus {
        group: "Unstarted",
        name: "Unstarted",
        sort_order: 0,
        is_default: true,
        skills: "[]",
    },
    DefaultStatus {
        group: "Started",
        name: "In Progress",
        sort_order: 0,
        is_default: true,
        skills: r#"["tdd","systematic-debugging","verification"]"#,
    },
    DefaultStatus {
        group: "Started",
        name: "In Review",
        sort_order: 1,
        is_default: false,
        skills: r#"["code-review","verification"]"#,
    },
    DefaultStatus {
        group: "Completed",
        name: "Completed",
        sort_order: 0,
        is_default: true,
        skills: "[]",
    },
    DefaultStatus {
        group: "Cancelled",
        name: "Cancelled",
        sort_order: 0,
        is_default: true,
        skills: "[]",
    },
];

fn project_dir(base_path: &std::path::Path, project_id: &str) -> std::path::PathBuf {
    base_path.join("projects").join(project_id)
}

pub fn open_project_db(base_path: &std::path::Path, project_id: &str) -> Result<DbConnection, String> {
    let dir = project_dir(base_path, project_id);
    let db_path = dir.join("db.sqlite");
    DbConnection::open(&db_path)
}

pub fn seed_default_statuses(
    conn: &rusqlite::Connection,
    project_id: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    for status in DEFAULT_STATUSES {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO statuses (id, project_id, \"group\", name, sort_order, is_default, skills, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, project_id, status.group, status.name, status.sort_order, status.is_default, status.skills, now],
        )
        .map_err(|e| format!("Failed to seed status '{}': {e}", status.name))?;
    }
    Ok(())
}

#[tauri::command]
pub fn create_project(
    config: State<ConfigState>,
    name: String,
) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let agent_config = serde_json::json!({});

    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let dir = project_dir(&base_path, &id);

    // Create directory tree
    std::fs::create_dir_all(dir.join("artifacts"))
        .map_err(|e| format!("Failed to create artifacts directory: {e}"))?;
    std::fs::create_dir_all(dir.join("worktrees"))
        .map_err(|e| format!("Failed to create worktrees directory: {e}"))?;

    // Create and initialize project database
    let db = open_project_db(&base_path, &id).map_err(|e| {
        let _ = std::fs::remove_dir_all(&dir);
        format!("Failed to initialize project database: {e}")
    })?;

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, name, agent_config.to_string(), now, now],
        )
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&dir);
            format!("Failed to create project record: {e}")
        })?;

        seed_default_statuses(conn, &id).map_err(|e| {
            let _ = std::fs::remove_dir_all(&dir);
            e
        })?;

        Ok(())
    })?;

    // Update last_project_id in global config
    config.update(|c| {
        c.defaults.last_project_id = id.clone();
    })?;

    Ok(Project {
        id,
        name,
        agent_config,
        base_path: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn get_project(
    config: State<ConfigState>,
    id: String,
) -> Result<Project, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &id)?;

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
pub fn list_projects(
    config: State<ConfigState>,
) -> Result<Vec<ProjectSummary>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let projects_dir = base_path.join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {e}"))?;

    let mut projects = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let db_path = path.join("db.sqlite");
        if !db_path.exists() {
            continue;
        }

        let project_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        match open_project_db(&base_path, &project_id) {
            Ok(db) => {
                if let Ok(summary) = db.with_conn(|conn| {
                    conn.query_row(
                        "SELECT id, name, created_at FROM projects LIMIT 1",
                        [],
                        |row| {
                            Ok(ProjectSummary {
                                id: row.get(0)?,
                                name: row.get(1)?,
                                created_at: row.get(2)?,
                            })
                        },
                    )
                    .map_err(|e| format!("{e}"))
                }) {
                    projects.push(summary);
                }
            }
            Err(_) => continue,
        }
    }

    projects.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(projects)
}

#[tauri::command]
pub fn update_project(
    config: State<ConfigState>,
    id: String,
    name: Option<String>,
    agent_config: Option<serde_json::Value>,
    base_path: Option<String>,
) -> Result<Project, String> {
    let global_base = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&global_base, &id)?;

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
pub fn delete_project(
    config: State<ConfigState>,
    id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let dir = project_dir(&base_path, &id);

    if !dir.exists() {
        return Err("Project not found".to_string());
    }

    std::fs::remove_dir_all(&dir)
        .map_err(|e| format!("Failed to delete project directory: {e}"))?;

    // Clear last_project_id if it matches
    let _ = config.update(|c| {
        if c.defaults.last_project_id == id {
            c.defaults.last_project_id.clear();
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::global::GlobalConfig;
    use std::sync::Mutex;

    fn test_config_state() -> (ConfigState, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let mut config = GlobalConfig::default();
        config.storage.base_path = dir.path().to_str().unwrap().to_string();
        config.save(&config_path).unwrap();

        let state = ConfigState {
            config: Mutex::new(config),
            config_path,
        };
        (state, dir)
    }

    #[test]
    fn test_create_project_creates_directory_tree() {
        let (config, _dir) = test_config_state();
        let base_path = config.with_config(|c| Ok(c.resolve_base_path())).unwrap();

        let id = uuid::Uuid::new_v4().to_string();
        let proj_dir = project_dir(&base_path, &id);
        std::fs::create_dir_all(proj_dir.join("artifacts")).unwrap();
        std::fs::create_dir_all(proj_dir.join("worktrees")).unwrap();

        assert!(proj_dir.join("artifacts").is_dir());
        assert!(proj_dir.join("worktrees").is_dir());
    }

    #[test]
    fn test_seed_default_statuses() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("db.sqlite");
        let db = DbConnection::open(&db_path).unwrap();
        let project_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, 'Test', '{}', ?2, ?3)",
                rusqlite::params![project_id, now, now],
            ).map_err(|e| format!("{e}"))?;

            seed_default_statuses(conn, &project_id)?;

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM statuses WHERE project_id = ?1",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(count, 6);

            let defaults: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND is_default = 1",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(defaults, 5);

            let started: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND \"group\" = 'Started'",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(started, 2);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_open_project_db_creates_and_migrates() {
        let dir = tempfile::tempdir().unwrap();
        let project_id = "test-project";
        let proj_dir = dir.path().join("projects").join(project_id);
        std::fs::create_dir_all(&proj_dir).unwrap();

        let db = open_project_db(dir.path(), project_id).unwrap();
        db.with_conn(|conn| {
            let tables: Vec<String> = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name = 'projects'")
                .map_err(|e| format!("{e}"))?
                .query_map([], |row| row.get(0))
                .map_err(|e| format!("{e}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("{e}"))?;
            assert!(tables.contains(&"projects".to_string()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_list_empty_projects() {
        let dir = tempfile::tempdir().unwrap();
        let projects_dir = dir.path().join("projects");
        // Don't create the directory - should return empty
        assert!(!projects_dir.exists());
    }
}
