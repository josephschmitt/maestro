use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::commands::projects::open_project_db;

use super::reattach::is_process_alive;
use super::{AgentRegistry, EventBus, MaestroEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCrashedEvent {
    pub workspace_id: String,
    pub project_id: String,
}

pub fn start_pid_monitor(
    app: AppHandle,
    event_bus: Option<Arc<EventBus>>,
    registry: Arc<AgentRegistry>,
    base_path: PathBuf,
) {
    tauri::async_runtime::spawn(async move {
        let mut tick = interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            check_running_pids(&app, event_bus.as_ref(), &registry, &base_path);
        }
    });
}

fn check_running_pids(
    app: &AppHandle,
    event_bus: Option<&Arc<EventBus>>,
    registry: &AgentRegistry,
    base_path: &PathBuf,
) {
    let projects_dir = base_path.join("projects");
    if !projects_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(entries) => entries,
        Err(_) => return,
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

        check_project_pids(app, event_bus, registry, base_path, &project_id);
    }
}

fn check_project_pids(
    app: &AppHandle,
    event_bus: Option<&Arc<EventBus>>,
    registry: &AgentRegistry,
    base_path: &PathBuf,
    project_id: &str,
) {
    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(_) => return,
    };

    let running = db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, pid FROM agent_workspaces WHERE status = 'running' AND pid IS NOT NULL",
            )
            .map_err(|e| format!("{e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| format!("{e}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{e}"))
    });

    let running = match running {
        Ok(r) => r,
        Err(_) => return,
    };

    for (workspace_id, pid) in running {
        let pid = pid as u32;
        if !is_process_alive(pid) {
            registry.remove(&workspace_id);

            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = db.with_conn(|conn| {
                conn.execute(
                    "UPDATE agent_workspaces SET status = 'failed', completed_at = ?1 WHERE id = ?2",
                    rusqlite::params![completed_at, workspace_id],
                )
                .map_err(|e| format!("{e}"))?;
                Ok(())
            });

            let event = AgentCrashedEvent {
                workspace_id: workspace_id.clone(),
                project_id: project_id.to_string(),
            };
            if let Some(bus) = event_bus {
                bus.emit_maestro(MaestroEvent::AgentCrashed(event.clone()));
                bus.emit_maestro(MaestroEvent::WorkspacesChanged {
                    project_id: project_id.to_string(),
                });
            }
            let _ = app.emit("agent-crashed", &event);
        }
    }
}
