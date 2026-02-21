use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;
use crate::executor::context::{assemble_context, CardInfo};
use crate::executor::lifecycle::{start_lifecycle_monitor, stop_agent_process};
use crate::executor::spawn::spawn_agent;
use crate::executor::stream::{start_stderr_streaming, start_stdin_forwarding, start_stdout_streaming};
use crate::executor::{AgentHandle, AgentRegistry};
use crate::ipc::server::IpcServer;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentWorkspace {
    pub id: String,
    pub card_id: String,
    pub agent_type: String,
    pub status: String,
    pub session_id: Option<String>,
    pub pid: Option<i64>,
    pub worktree_path: Option<String>,
    pub branch_name: Option<String>,
    pub review_count: i32,
    pub attached_at: String,
    pub completed_at: Option<String>,
}

fn row_to_workspace(row: &rusqlite::Row) -> Result<AgentWorkspace, rusqlite::Error> {
    Ok(AgentWorkspace {
        id: row.get(0)?,
        card_id: row.get(1)?,
        agent_type: row.get(2)?,
        status: row.get(3)?,
        session_id: row.get(4)?,
        pid: row.get(5)?,
        worktree_path: row.get(6)?,
        branch_name: row.get(7)?,
        review_count: row.get(8)?,
        attached_at: row.get(9)?,
        completed_at: row.get(10)?,
    })
}

const WORKSPACE_SELECT: &str = "\
    SELECT id, card_id, agent_type, status, session_id, pid, worktree_path, \
           branch_name, review_count, attached_at, completed_at \
    FROM agent_workspaces";

fn collect_artifact_contents(artifacts_dir: &std::path::Path) -> Vec<(String, String)> {
    let mut contents = Vec::new();
    if !artifacts_dir.exists() {
        return contents;
    }
    if let Ok(entries) = std::fs::read_dir(artifacts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    contents.push((name, content));
                }
            }
        }
    }
    contents
}

#[tauri::command]
pub async fn launch_agent(
    app: AppHandle,
    config: State<'_, ConfigState>,
    registry: State<'_, Arc<AgentRegistry>>,
    project_id: String,
    card_id: String,
    status_group: String,
    worktree_path: Option<String>,
    branch_name: Option<String>,
) -> Result<AgentWorkspace, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    let (card_title, card_description, parent_title, parent_description, project_agent_config) =
        db.with_conn(|conn| {
            let (title, description, parent_id): (String, String, Option<String>) = conn
                .query_row(
                    "SELECT c.title, c.description, c.parent_id FROM cards c WHERE c.id = ?1 AND c.project_id = ?2",
                    rusqlite::params![card_id, project_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|e| format!("Card not found: {e}"))?;

            let (parent_title, parent_description) = if let Some(ref pid) = parent_id {
                let result: Result<(String, String), _> = conn.query_row(
                    "SELECT title, description FROM cards WHERE id = ?1",
                    rusqlite::params![pid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                );
                match result {
                    Ok((t, d)) => (Some(t), Some(d)),
                    Err(_) => (None, None),
                }
            } else {
                (None, None)
            };

            let agent_config_json: String = conn
                .query_row(
                    "SELECT agent_config FROM projects WHERE id = ?1",
                    rusqlite::params![project_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Project not found: {e}"))?;

            let project_agent_config: serde_json::Value =
                serde_json::from_str(&agent_config_json).unwrap_or_default();

            Ok((title, description, parent_title, parent_description, project_agent_config))
        })?;

    let card_info = CardInfo {
        id: card_id.clone(),
        title: card_title,
        description: card_description,
        parent_title,
        parent_description,
    };

    let working_dir = if let Some(ref wt) = worktree_path {
        std::path::PathBuf::from(wt)
    } else {
        base_path
            .join("projects")
            .join(&project_id)
            .join("artifacts")
            .join(&card_id)
    };

    let working_dir_str = working_dir.to_string_lossy().to_string();

    let artifacts_dir = base_path
        .join("projects")
        .join(&project_id)
        .join("artifacts")
        .join(&card_id);

    let artifact_contents = if worktree_path.is_some() {
        collect_artifact_contents(&artifacts_dir)
    } else {
        Vec::new()
    };

    let socket_path = IpcServer::socket_path(&project_id);
    let socket_path_str = if socket_path.exists() {
        Some(socket_path.to_string_lossy().to_string())
    } else {
        None
    };

    let agent_ctx = config.with_config(|c| {
        assemble_context(
            c,
            &project_agent_config,
            &status_group,
            &card_info,
            &working_dir_str,
            &artifact_contents,
            socket_path_str.as_deref(),
        )
    })?;

    let mut spawned = spawn_agent(&agent_ctx)?;

    let stdout = spawned.child.stdout.take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = spawned.child.stderr.take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;
    let stdin = spawned.child.stdin.take()
        .ok_or_else(|| "Failed to capture stdin".to_string())?;

    let workspace_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let pid = spawned.pid;

    let workspace = db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO agent_workspaces (id, card_id, agent_type, status, pid, worktree_path, branch_name, attached_at) \
             VALUES (?1, ?2, ?3, 'running', ?4, ?5, ?6, ?7)",
            rusqlite::params![workspace_id, card_id, agent_ctx.binary, pid as i64, worktree_path, branch_name, now],
        )
        .map_err(|e| format!("Failed to create workspace: {e}"))?;

        conn.query_row(
            &format!("{WORKSPACE_SELECT} WHERE id = ?1"),
            rusqlite::params![workspace_id],
            row_to_workspace,
        )
        .map_err(|e| format!("Failed to read workspace: {e}"))
    })?;

    start_stdout_streaming(app.clone(), workspace_id.clone(), stdout);
    start_stderr_streaming(app.clone(), workspace_id.clone(), stderr);

    let (stdin_tx, stdin_rx) = tokio::sync::mpsc::channel::<String>(64);
    start_stdin_forwarding(stdin, stdin_rx);

    let handle = AgentHandle {
        workspace_id: workspace_id.clone(),
        stdin_tx,
        pid,
    };
    registry.insert(handle);

    start_lifecycle_monitor(
        app,
        Arc::clone(&registry),
        spawned.child,
        workspace_id,
        project_id,
        base_path,
    );

    Ok(workspace)
}

#[tauri::command]
pub async fn send_agent_input(
    registry: State<'_, Arc<AgentRegistry>>,
    workspace_id: String,
    text: String,
) -> Result<(), String> {
    let tx = registry
        .get_stdin_tx(&workspace_id)
        .ok_or_else(|| format!("No running agent for workspace {workspace_id}"))?;

    tx.send(text)
        .await
        .map_err(|e| format!("Failed to send input: {e}"))
}

#[tauri::command]
pub async fn stop_agent(
    config: State<'_, ConfigState>,
    registry: State<'_, Arc<AgentRegistry>>,
    project_id: String,
    workspace_id: String,
) -> Result<AgentWorkspace, String> {
    let handle = registry
        .remove(&workspace_id)
        .ok_or_else(|| format!("No running agent for workspace {workspace_id}"))?;

    stop_agent_process(handle.pid).await?;

    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    let completed_at = chrono::Utc::now().to_rfc3339();
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE agent_workspaces SET status = 'failed', completed_at = ?1 WHERE id = ?2",
            rusqlite::params![completed_at, workspace_id],
        )
        .map_err(|e| format!("Failed to update workspace: {e}"))?;

        conn.query_row(
            &format!("{WORKSPACE_SELECT} WHERE id = ?1"),
            rusqlite::params![workspace_id],
            row_to_workspace,
        )
        .map_err(|e| format!("Failed to read workspace: {e}"))
    })
}

#[tauri::command]
pub fn list_workspaces(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<Vec<AgentWorkspace>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(&format!(
                "{WORKSPACE_SELECT} WHERE card_id = ?1 ORDER BY attached_at DESC"
            ))
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![card_id], |row| row_to_workspace(row))
            .map_err(|e| format!("Failed to query workspaces: {e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read workspace row: {e}"))
    })
}

#[tauri::command]
pub fn get_workspace(
    config: State<ConfigState>,
    project_id: String,
    workspace_id: String,
) -> Result<AgentWorkspace, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        conn.query_row(
            &format!("{WORKSPACE_SELECT} WHERE id = ?1"),
            rusqlite::params![workspace_id],
            row_to_workspace,
        )
        .map_err(|e| format!("Workspace not found: {e}"))
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
             VALUES (?1, ?2, ?3, 'Test Card', 'Test description', '[]', 0, ?4, ?5)",
            rusqlite::params![id, project_id, status_id, now, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_create_and_list_workspaces() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let ws_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO agent_workspaces (id, card_id, agent_type, status, pid, attached_at) \
                 VALUES (?1, ?2, 'claude', 'running', 12345, ?3)",
                rusqlite::params![ws_id, card_id, now],
            )
            .map_err(|e| format!("{e}"))?;

            let mut stmt = conn
                .prepare(&format!("{WORKSPACE_SELECT} WHERE card_id = ?1"))
                .map_err(|e| format!("{e}"))?;

            let workspaces: Vec<AgentWorkspace> = stmt
                .query_map(rusqlite::params![card_id], |row| row_to_workspace(row))
                .map_err(|e| format!("{e}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("{e}"))?;

            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].id, ws_id);
            assert_eq!(workspaces[0].status, "running");
            assert_eq!(workspaces[0].pid, Some(12345));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_workspace_status_update() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id);
            let ws_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO agent_workspaces (id, card_id, agent_type, status, pid, attached_at) \
                 VALUES (?1, ?2, 'claude', 'running', 12345, ?3)",
                rusqlite::params![ws_id, card_id, now],
            )
            .map_err(|e| format!("{e}"))?;

            conn.execute(
                "UPDATE agent_workspaces SET status = 'completed', completed_at = ?1 WHERE id = ?2",
                rusqlite::params![now, ws_id],
            )
            .map_err(|e| format!("{e}"))?;

            let workspace = conn
                .query_row(
                    &format!("{WORKSPACE_SELECT} WHERE id = ?1"),
                    rusqlite::params![ws_id],
                    row_to_workspace,
                )
                .map_err(|e| format!("{e}"))?;

            assert_eq!(workspace.status, "completed");
            assert!(workspace.completed_at.is_some());
            Ok(())
        })
        .unwrap();
    }
}
