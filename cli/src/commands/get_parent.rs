use crate::ipc::{send_request, IpcRequest};

pub async fn run(socket_path: &str, card_id: &str) -> Result<String, String> {
    let request = IpcRequest {
        command: "get-parent".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({}),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        let data = response.data.unwrap_or(serde_json::json!(null));
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to format response: {e}"))
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
