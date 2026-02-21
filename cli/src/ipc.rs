use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    pub command: String,
    pub card_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn send_request(socket_path: &str, request: IpcRequest) -> Result<IpcResponse, String> {
    if !Path::new(socket_path).exists() {
        return Err(format!(
            "Socket not found at {socket_path}. Is the Maestro app running?"
        ));
    }

    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to Maestro socket: {e}"))?;

    let (reader, mut writer) = stream.into_split();

    let mut json = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize request: {e}"))?;
    json.push('\n');

    writer
        .write_all(json.as_bytes())
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;

    writer
        .shutdown()
        .await
        .map_err(|e| format!("Failed to shutdown write half: {e}"))?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    buf_reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if response_line.is_empty() {
        return Err("Empty response from server".to_string());
    }

    serde_json::from_str(&response_line)
        .map_err(|e| format!("Failed to parse response: {e}"))
}
