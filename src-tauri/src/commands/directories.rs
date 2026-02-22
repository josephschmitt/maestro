use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;
use crate::fs::git::is_git_repo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkedDirectory {
    pub id: String,
    pub project_id: String,
    pub path: String,
    pub label: String,
    pub is_repo: bool,
    pub created_at: String,
}

fn row_to_linked_directory(row: &rusqlite::Row) -> Result<LinkedDirectory, rusqlite::Error> {
    let is_repo_int: i32 = row.get(4)?;
    Ok(LinkedDirectory {
        id: row.get(0)?,
        project_id: row.get(1)?,
        path: row.get(2)?,
        label: row.get(3)?,
        is_repo: is_repo_int != 0,
        created_at: row.get(5)?,
    })
}

const DIR_SELECT: &str = "\
    SELECT id, project_id, path, label, is_repo, created_at \
    FROM linked_directories";

pub fn add_linked_directory_inner(
    config: &ConfigState,
    project_id: &str,
    path: &str,
    label: &str,
) -> Result<LinkedDirectory, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let project_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify project: {e}"))?;

        if !project_exists {
            return Err(format!("Project {project_id} not found"));
        }

        let trimmed_path = path.trim().to_string();
        if trimmed_path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        let trimmed_label = label.trim().to_string();
        if trimmed_label.is_empty() {
            return Err("Label cannot be empty".to_string());
        }

        let duplicate_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM linked_directories WHERE project_id = ?1 AND path = ?2",
                rusqlite::params![project_id, trimmed_path],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check for duplicate: {e}"))?;

        if duplicate_count > 0 {
            return Err(format!("Directory already linked: {trimmed_path}"));
        }

        let is_repo = is_git_repo(&trimmed_path);
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO linked_directories (id, project_id, path, label, is_repo, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, project_id, trimmed_path, trimmed_label, is_repo as i32, now],
        )
        .map_err(|e| format!("Failed to add linked directory: {e}"))?;

        conn.query_row(
            &format!("{DIR_SELECT} WHERE id = ?1"),
            rusqlite::params![id],
            row_to_linked_directory,
        )
        .map_err(|e| format!("Failed to read created linked directory: {e}"))
    })
}

#[tauri::command]
pub fn add_linked_directory(
    config: State<ConfigState>,
    project_id: String,
    path: String,
    label: String,
) -> Result<LinkedDirectory, String> {
    add_linked_directory_inner(&config, &project_id, &path, &label)
}

pub fn remove_linked_directory_inner(
    config: &ConfigState,
    project_id: &str,
    id: &str,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let rows_affected = conn
            .execute(
                "DELETE FROM linked_directories WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![id, project_id],
            )
            .map_err(|e| format!("Failed to remove linked directory: {e}"))?;

        if rows_affected == 0 {
            return Err(format!("Linked directory {id} not found"));
        }

        Ok(())
    })
}

#[tauri::command]
pub fn remove_linked_directory(
    config: State<ConfigState>,
    project_id: String,
    id: String,
) -> Result<(), String> {
    remove_linked_directory_inner(&config, &project_id, &id)
}

pub fn list_linked_directories_inner(
    config: &ConfigState,
    project_id: &str,
) -> Result<Vec<LinkedDirectory>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{DIR_SELECT} WHERE project_id = ?1 ORDER BY created_at ASC"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![project_id], |row| {
                row_to_linked_directory(row)
            })
            .map_err(|e| format!("Failed to query linked directories: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read linked directory row: {e}"))
    })
}

#[tauri::command]
pub fn list_linked_directories(
    config: State<ConfigState>,
    project_id: String,
) -> Result<Vec<LinkedDirectory>, String> {
    list_linked_directories_inner(&config, &project_id)
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

    fn insert_linked_dir(conn: &rusqlite::Connection, project_id: &str, path: &str, label: &str, is_repo: bool) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO linked_directories (id, project_id, path, label, is_repo, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, project_id, path, label, is_repo as i32, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_add_and_list_linked_directories() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            insert_linked_dir(conn, &project_id, "/home/user/repo1", "Repo 1", true);
            insert_linked_dir(conn, &project_id, "/home/user/docs", "Docs", false);

            let mut stmt = conn
                .prepare(&format!("{DIR_SELECT} WHERE project_id = ?1 ORDER BY created_at ASC"))
                .unwrap();
            let dirs: Vec<LinkedDirectory> = stmt
                .query_map(rusqlite::params![project_id], |row| row_to_linked_directory(row))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(dirs.len(), 2);
            assert_eq!(dirs[0].label, "Repo 1");
            assert!(dirs[0].is_repo);
            assert_eq!(dirs[1].label, "Docs");
            assert!(!dirs[1].is_repo);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_remove_linked_directory() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let id = insert_linked_dir(conn, &project_id, "/home/user/repo", "Repo", true);

            conn.execute(
                "DELETE FROM linked_directories WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM linked_directories WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_cascade_delete_on_project_removal() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            insert_linked_dir(conn, &project_id, "/home/user/repo1", "R1", true);
            insert_linked_dir(conn, &project_id, "/home/user/repo2", "R2", false);

            conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![project_id])
                .unwrap();

            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM linked_directories WHERE project_id = ?1",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }
}
