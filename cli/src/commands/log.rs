use crate::ipc::{send_request, IpcRequest};

pub async fn run(socket_path: &str, card_id: &str, message: &str) -> Result<String, String> {
    let request = IpcRequest {
        command: "log".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({ "message": message }),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        Ok(String::new())
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
