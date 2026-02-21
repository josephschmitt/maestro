use crate::ipc::{send_request, IpcRequest};

pub async fn run(
    socket_path: &str,
    card_id: &str,
    question_id: &str,
    resolution: Option<&str>,
) -> Result<String, String> {
    let request = IpcRequest {
        command: "resolve-question".to_string(),
        card_id: card_id.to_string(),
        payload: serde_json::json!({
            "id": question_id,
            "resolution": resolution,
        }),
    };

    let response = send_request(socket_path, request).await?;

    if response.ok {
        Ok(format!("Question {question_id} resolved"))
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
