use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::commands::config::ConfigState;
use crate::ipc::server::IpcServer;

#[tauri::command]
pub async fn start_ipc_server(
    app: AppHandle,
    config: State<'_, ConfigState>,
    ipc_server: State<'_, Arc<IpcServer>>,
    project_id: String,
) -> Result<String, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;

    let socket_path = ipc_server
        .start(app, base_path, project_id)
        .await?;

    Ok(socket_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn stop_ipc_server(
    ipc_server: State<'_, Arc<IpcServer>>,
    project_id: String,
) -> Result<(), String> {
    ipc_server.stop_project(&project_id).await;
    Ok(())
}
