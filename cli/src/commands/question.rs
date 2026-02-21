use crate::ipc::{send_request, IpcRequest};

pub async fn run(socket_path: &str, card_id: &str, question: &str) -> Result<String, String> {
    let request = IpcRequest {
        command: "question".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({ "question": question }),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        let id = response
            .data
            .as_ref()
            .and_then(|d| d.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(format!("Question created: {id}"))
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
