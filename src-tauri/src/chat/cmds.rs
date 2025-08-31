use futures_util::StreamExt;
use serde_json::json;
use tauri::AppHandle;
use tauri::Emitter;
use uuid::Uuid;

use crate::common::agent::GeminiAgent;
use crate::common::errors::AppError;
use crate::common::http::BuildAgentPromptRequest;
use crate::common::http::HttpClientManager;

#[tauri::command]
pub fn create_chat() -> Uuid {
    Uuid::new_v4()
}

#[tauri::command]
pub async fn send_chat_message(
    app_handle: AppHandle,
    client_manager: tauri::State<'_, HttpClientManager>,
    chat_id: Uuid,
    content: String,
) -> Result<Uuid, AppError> {
    let client = client_manager.get_client();
    let body = json!({
        "contents": [
            {
                "parts": [
                    {
                        "role": "user",
                        "text": content
                    }
                ]
            }
        ]
    });

    // mock agent
    let agent = GeminiAgent {
        id: "".into(),
        api_key: "".into(),
        model: "gemini-2.5-flash".into(),
    };
    let response = agent
        .build_prompt_request(client)
        .json(&body)
        .send()
        .await?;

    let chat_id = Uuid::new_v4();
    tauri::async_runtime::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        for line in text.lines() {
                            if line.starts_with("data:") {
                                let data = line[5..].trim();
                                let _ = app_handle.emit("agent-response-chunk", data).inspect_err(
                                    |e| {
                                        log::error!("Failed to emit response chunk: {e}");
                                    },
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    log::error!("Stream error: {err}");
                    break;
                }
            }
        }
    });
    Ok(chat_id)
}
