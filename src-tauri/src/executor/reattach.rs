use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::commands::projects::open_project_db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReattachResult {
    pub reattached: Vec<ReattachedWorkspace>,
    pub failed: Vec<FailedWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReattachedWorkspace {
    pub workspace_id: String,
    pub project_id: String,
    pub card_id: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedWorkspace {
    pub workspace_id: String,
    pub project_id: String,
    pub card_id: String,
    pub session_id: Option<String>,
}

pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let result = unsafe { libc::kill(pid as i32, 0) };
        result == 0
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

pub fn startup_scan(app: &AppHandle, base_path: &Path) -> ReattachResult {
    let mut result = ReattachResult {
        reattached: Vec::new(),
        failed: Vec::new(),
    };

    let projects_dir = base_path.join("projects");
    if !projects_dir.exists() {
        return result;
    }

    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(entries) => entries,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let db_path = path.join("db.sqlite");
        if !db_path.exists() {
            continue;
        }

        let project_id = match path.file_name().and_then(|n| n.to_str()) {
            Some(id) => id.to_string(),
            None => continue,
        };

        scan_project(app, base_path, &project_id, &mut result);
    }

    if !result.failed.is_empty() {
        let _ = app.emit("agents-crashed-on-startup", &result.failed);
    }

    result
}

fn scan_project(
    _app: &AppHandle,
    base_path: &Path,
    project_id: &str,
    result: &mut ReattachResult,
) {
    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(_) => return,
    };

    let running_workspaces = db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, card_id, pid, session_id FROM agent_workspaces WHERE status = 'running'",
            )
            .map_err(|e| format!("Failed to query running workspaces: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|e| format!("Failed to read workspaces: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect workspaces: {e}"))
    });

    let running_workspaces = match running_workspaces {
        Ok(ws) => ws,
        Err(_) => return,
    };

    for (workspace_id, card_id, pid, session_id) in running_workspaces {
        let pid = match pid {
            Some(p) if p > 0 => p as u32,
            _ => {
                mark_workspace_failed(&db, &workspace_id);
                result.failed.push(FailedWorkspace {
                    workspace_id,
                    project_id: project_id.to_string(),
                    card_id,
                    session_id,
                });
                continue;
            }
        };

        if is_process_alive(pid) {
            result.reattached.push(ReattachedWorkspace {
                workspace_id,
                project_id: project_id.to_string(),
                card_id,
                pid,
            });
        } else {
            mark_workspace_failed(&db, &workspace_id);
            result.failed.push(FailedWorkspace {
                workspace_id,
                project_id: project_id.to_string(),
                card_id,
                session_id,
            });
        }
    }
}

fn mark_workspace_failed(db: &crate::db::DbConnection, workspace_id: &str) {
    let completed_at = chrono::Utc::now().to_rfc3339();
    let _ = db.with_conn(|conn| {
        conn.execute(
            "UPDATE agent_workspaces SET status = 'failed', completed_at = ?1 WHERE id = ?2",
            rusqlite::params![completed_at, workspace_id],
        )
        .map_err(|e| format!("Failed to mark workspace as failed: {e}"))?;
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_alive_self() {
        let pid = std::process::id();
        assert!(is_process_alive(pid));
    }

    #[test]
    fn test_is_process_alive_nonexistent() {
        assert!(!is_process_alive(99999));
    }
}
