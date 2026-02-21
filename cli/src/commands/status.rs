use crate::ipc::{send_request, IpcRequest};

pub async fn run(socket_path: &str, card_id: &str, status: &str) -> Result<String, String> {
    let request = IpcRequest {
        command: "set-status".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({ "status": status }),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        Ok(format!("Status set to: {status}"))
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
