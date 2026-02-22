use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Child;
use tokio::time::Duration;

use crate::commands::projects::open_project_db;

use super::{AgentEvent, AgentRegistry, EventBus, MaestroEvent};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExitEvent {
    pub workspace_id: String,
    pub exit_code: Option<i32>,
    pub status: String,
}

pub fn start_lifecycle_monitor(
    app: AppHandle,
    registry: Arc<AgentRegistry>,
    child: Child,
    workspace_id: String,
    project_id: String,
    base_path: PathBuf,
) {
    start_lifecycle_monitor_inner(Some(app), None, registry, child, workspace_id, project_id, base_path);
}

pub fn start_lifecycle_monitor_inner(
    app: Option<AppHandle>,
    event_bus: Option<Arc<EventBus>>,
    registry: Arc<AgentRegistry>,
    mut child: Child,
    workspace_id: String,
    project_id: String,
    base_path: PathBuf,
) {
    tokio::spawn(async move {
        let exit_status = child.wait().await;
        let (exit_code, status) = match exit_status {
            Ok(status) => {
                let code = status.code();
                let s = if code == Some(0) {
                    "completed"
                } else {
                    "failed"
                };
                (code, s.to_string())
            }
            Err(_) => (None, "failed".to_string()),
        };

        registry.remove(&workspace_id);

        if let Ok(db) = open_project_db(&base_path, &project_id) {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = db.with_conn(|conn| {
                conn.execute(
                    "UPDATE agent_workspaces SET status = ?1, completed_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, completed_at, workspace_id],
                )
                .map_err(|e| format!("Failed to update workspace status: {e}"))?;
                Ok(())
            });
        }

        let event = AgentExitEvent {
            workspace_id: workspace_id.clone(),
            exit_code,
            status: status.clone(),
        };
        if let Some(ref bus) = event_bus {
            bus.emit(AgentEvent::Exit(event.clone()));
            bus.emit_maestro(MaestroEvent::AgentExit(event.clone()));
            bus.emit_maestro(MaestroEvent::WorkspacesChanged {
                project_id: project_id.clone(),
            });
        }
        if let Some(ref handle) = app {
            let _ = handle.emit(&format!("agent-exit-{}", workspace_id), &event);
        }
    });
}

pub async fn stop_agent_process(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }

        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(5) {
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
                return Ok(());
            }
            let result = unsafe { libc::kill(pid as i32, 0) };
            if result != 0 {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    #[cfg(not(unix))]
    {
        Err("Stop not supported on this platform".to_string())
    }
}
