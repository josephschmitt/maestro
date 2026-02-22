use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::ChildStderr;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::sync::mpsc;

use serde::{Deserialize, Serialize};

use super::{AgentEvent, EventBus, MaestroEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutputEvent {
    pub workspace_id: String,
    pub stream: String,
    pub line: String,
}

pub fn start_stdout_streaming(
    app: AppHandle,
    workspace_id: String,
    stdout: ChildStdout,
) {
    start_stdout_streaming_inner(Some(app), None, workspace_id, stdout);
}

pub fn start_stdout_streaming_inner(
    app: Option<AppHandle>,
    event_bus: Option<Arc<EventBus>>,
    workspace_id: String,
    stdout: ChildStdout,
) {
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let event = AgentOutputEvent {
                workspace_id: workspace_id.clone(),
                stream: "stdout".to_string(),
                line,
            };
            if let Some(ref bus) = event_bus {
                bus.emit(AgentEvent::Output(event.clone()));
                bus.emit_maestro(MaestroEvent::AgentOutput(event.clone()));
            }
            if let Some(ref handle) = app {
                let _ = handle.emit(&format!("agent-output-{}", workspace_id), &event);
            }
        }
    });
}

pub fn start_stderr_streaming(
    app: AppHandle,
    workspace_id: String,
    stderr: ChildStderr,
) {
    start_stderr_streaming_inner(Some(app), None, workspace_id, stderr);
}

pub fn start_stderr_streaming_inner(
    app: Option<AppHandle>,
    event_bus: Option<Arc<EventBus>>,
    workspace_id: String,
    stderr: ChildStderr,
) {
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let event = AgentOutputEvent {
                workspace_id: workspace_id.clone(),
                stream: "stderr".to_string(),
                line,
            };
            if let Some(ref bus) = event_bus {
                bus.emit(AgentEvent::Output(event.clone()));
                bus.emit_maestro(MaestroEvent::AgentOutput(event.clone()));
            }
            if let Some(ref handle) = app {
                let _ = handle.emit(&format!("agent-output-{}", workspace_id), &event);
            }
        }
    });
}

pub fn start_stdin_forwarding(
    mut stdin: ChildStdin,
    mut rx: mpsc::Receiver<String>,
) {
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            let data = format!("{line}\n");
            if stdin.write_all(data.as_bytes()).await.is_err() {
                break;
            }
            if stdin.flush().await.is_err() {
                break;
            }
        }
    });
}
