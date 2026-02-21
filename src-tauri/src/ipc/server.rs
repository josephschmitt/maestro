use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use crate::ipc::handler::handle_request;
use crate::ipc::protocol::{IpcRequest, IpcResponse};

pub struct IpcServer {
    socket_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl IpcServer {
    pub fn new() -> Self {
        Self {
            socket_paths: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn socket_path(project_id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/maestro-{project_id}.sock"))
    }

    pub async fn start(
        &self,
        app: AppHandle,
        base_path: PathBuf,
        project_id: String,
    ) -> Result<PathBuf, String> {
        let socket_path = Self::socket_path(&project_id);

        // Clean up any stale socket file
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .map_err(|e| format!("Failed to remove stale socket: {e}"))?;
        }

        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("Failed to bind Unix socket at {}: {e}", socket_path.display()))?;

        self.socket_paths.lock().await.push(socket_path.clone());

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            accept_loop(listener, app, base_path, project_id).await;
            // Clean up socket file when the loop exits
            let _ = std::fs::remove_file(&socket_path_clone);
        });

        Ok(socket_path)
    }

    pub async fn stop_all(&self) {
        let paths = self.socket_paths.lock().await;
        for path in paths.iter() {
            let _ = std::fs::remove_file(path);
        }
    }

    pub async fn stop_project(&self, project_id: &str) {
        let socket_path = Self::socket_path(project_id);
        let _ = std::fs::remove_file(&socket_path);
        let mut paths = self.socket_paths.lock().await;
        paths.retain(|p| p != &socket_path);
    }
}

async fn accept_loop(
    listener: UnixListener,
    app: AppHandle,
    base_path: PathBuf,
    project_id: String,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let app = app.clone();
                let base_path = base_path.clone();
                let project_id = project_id.clone();

                tokio::spawn(async move {
                    handle_connection(stream, &app, &base_path, &project_id).await;
                });
            }
            Err(e) => {
                // Socket was likely removed (server shutting down)
                if is_fatal_accept_error(&e) {
                    break;
                }
                eprintln!("IPC accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    app: &AppHandle,
    base_path: &Path,
    project_id: &str,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    match buf_reader.read_line(&mut line).await {
        Ok(0) => return, // EOF
        Ok(_) => {}
        Err(e) => {
            eprintln!("IPC read error: {e}");
            return;
        }
    }

    let response = match serde_json::from_str::<IpcRequest>(&line) {
        Ok(request) => handle_request(app, &base_path.to_path_buf(), project_id, request),
        Err(e) => IpcResponse::error(format!("Invalid request JSON: {e}")),
    };

    let mut response_json =
        serde_json::to_string(&response).unwrap_or_else(|_| r#"{"ok":false,"error":"Failed to serialize response"}"#.to_string());
    response_json.push('\n');

    if let Err(e) = writer.write_all(response_json.as_bytes()).await {
        eprintln!("IPC write error: {e}");
    }
}

fn is_fatal_accept_error(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::NotFound
            | std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::InvalidInput
    )
}
