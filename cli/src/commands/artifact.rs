use crate::ipc::{send_request, IpcRequest};
use std::path::Path;

pub async fn run(
    socket_path: &str,
    card_id: &str,
    file_path: &str,
    name: Option<&str>,
) -> Result<String, String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("File not found: {file_path}"));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let display_name = name
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("artifact")
                .to_string()
        });

    let request = IpcRequest {
        command: "add-artifact".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({
            "name": display_name,
            "content": content,
        }),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        let id = response
            .data
            .as_ref()
            .and_then(|d| d.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(format!("Artifact added: {id}"))
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
