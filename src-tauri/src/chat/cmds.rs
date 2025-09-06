use std::sync::Arc;

use futures_util::StreamExt;
use serde::Serialize;
use slug::slugify;
use sqlx::Pool;
use sqlx::Sqlite;
use tauri::AppHandle;
use tauri::Emitter;
use uuid::Uuid;

use crate::chat::utils::get_chat_messages;
use crate::chat::utils::{
    create_chat_message, update_chat_message, CreateChatMessage, UpdateChatMessage,
};
use crate::common::agent::{AgentApi, AgentContext};
use crate::common::agent_gemini::{
    GeminiAgent, GeminiTextGenRequestParams, GeminiTextGenRequestParamsMessage,
};
use crate::common::entity::chat::ChatMessageStatus;
use crate::common::errors::AppError;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageResponseChunkPayload {
    pub chat_id: Uuid,
    pub id: Uuid,
    pub text: String,
}

#[tauri::command]
pub async fn create_chat(
    content: String,
    db_pool: tauri::State<'_, Arc<Pool<Sqlite>>>,
) -> Result<Uuid, AppError> {
    let chat_id = Uuid::new_v4();
    let title = slugify(content);
    sqlx::query("insert into chats (id, title) values ($1, $2)")
        .bind(chat_id)
        .bind(title)
        .execute(&**db_pool.inner())
        .await
        .map_err(|e| AppError::Unknown(Some(Box::new(e))))?;
    Ok(chat_id)
}

#[tauri::command]
pub async fn send_chat_message(
    chat_id: Uuid,
    content: String,
    app_handle: AppHandle,
    agent_context: tauri::State<'_, AgentContext<Sqlite>>,
    db_pool: tauri::State<'_, Arc<Pool<Sqlite>>>,
) -> Result<(), AppError> {
    let agent = GeminiAgent {
        id: "".into(),
        model: "gemini-2.5-flash".into(),
    };
    let user_chat_msg = create_chat_message(
        CreateChatMessage {
            id: Uuid::new_v4(),
            chat_id,
            role: "user".into(),
            content: content.clone(),
            status: ChatMessageStatus::Completed,
        },
        db_pool.inner(),
    )
    .await?;
    let _ = app_handle
        .emit("chat_message_created", &user_chat_msg)
        .inspect_err(|e| {
            log::error!("Failed to emit response chunk: {e}");
        });

    let chat_messages = get_chat_messages(chat_id, db_pool.inner()).await?;
    let mut stream = agent
        .generate_text(
            agent_context.inner().clone(),
            GeminiTextGenRequestParams {
                api_key: "".into(),
                messages: chat_messages
                    .into_iter()
                    .map(|msg| GeminiTextGenRequestParamsMessage {
                        role: msg.role,
                        content: msg.content,
                    })
                    .collect(),
            },
        )
        .await?;

    let model_chat_msg = create_chat_message(
        CreateChatMessage {
            id: Uuid::new_v4(),
            chat_id,
            role: "model".into(),
            content: String::new(),
            status: ChatMessageStatus::Pending,
        },
        db_pool.inner(),
    )
    .await?;
    let _ = app_handle
        .emit("chat_message_created", model_chat_msg.clone())
        .inspect_err(|e| {
            log::error!("Failed to emit response chunk: {e}");
        });

    let db_pool_clone = db_pool.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut text = String::new();
        let mut chunk_count = 0;
        while let Some(item) = stream.next().await {
            match item {
                Ok(result) => {
                    text.push_str(&result.text);
                    chunk_count += 1;
                    if chunk_count == 5 {
                        chunk_count = 0;
                        let _ = update_chat_message(
                            model_chat_msg.id,
                            UpdateChatMessage {
                                content: Some(text.clone()),
                                ..Default::default()
                            },
                            &*db_pool_clone,
                        )
                        .await
                        .inspect_err(|e| {
                            log::error!("failed to update chat message content: {e}");
                        });
                    }
                    let _ = app_handle
                        .emit(
                            "chat_message_response_chunk",
                            ChatMessageResponseChunkPayload {
                                chat_id,
                                id: model_chat_msg.id,
                                text: result.text,
                            },
                        )
                        .inspect_err(|e| {
                            log::error!("failed to emit response chunk: {e}");
                        });
                }
                Err(err) => {
                    log::error!("Stream error: {err}");
                    let _ = update_chat_message(
                        model_chat_msg.id,
                        UpdateChatMessage {
                            content: Some(text.clone()),
                            status: Some(ChatMessageStatus::Failed),
                            ..Default::default()
                        },
                        &*db_pool_clone,
                    )
                    .await
                    .inspect_err(|e| {
                        log::error!("failed to update chat message status and content: {e}");
                    });
                    return;
                }
            }
        }
        let _ = update_chat_message(
            model_chat_msg.id,
            UpdateChatMessage {
                content: match chunk_count {
                    0 => None,
                    _ => Some(text.clone()),
                },
                status: Some(ChatMessageStatus::Completed),
                ..Default::default()
            },
            &*db_pool_clone,
        )
        .await
        .inspect_err(|e| {
            log::error!("failed to update chat message status and content: {e}");
        });
    });

    Ok(())
}
