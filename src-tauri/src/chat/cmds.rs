use futures_util::StreamExt;
use serde::Serialize;
use slug::slugify;
use sqlx::Pool;
use sqlx::Sqlite;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;
use uuid::Uuid;

use crate::common::entity::chat::ChatMessageRow;
use crate::common::entity::chat::ChatRow;
use crate::{
    agent::{Agent, AgentApi, AgentContext, AgentTextGenParamsApi},
    chat::repo::{ChatRepo, CreateChatMessage, UpdateChatMessage},
    common::{entity::chat::ChatMessageStatus, error::AppError, unit_of_work::UnitOfWorkFactory},
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageResponseChunkPayload {
    pub chat_id: Uuid,
    pub id: Uuid,
    pub text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRollbackPayload {
    pub chat_id: Uuid,
    pub message_id: Uuid,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageStatusChangedPayload {
    pub chat_id: Uuid,
    pub message_id: Uuid,
    pub status: ChatMessageStatus,
}

#[tauri::command]
pub async fn create_chat(
    content: String,
    db_pool: tauri::State<'_, Arc<Pool<Sqlite>>>,
) -> Result<Uuid, AppError> {
    let chat_id = Uuid::new_v4();
    let title = slugify(content);
    sqlx::query("insert into chats (id, title) values (?1, ?2)")
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
    agent_context: tauri::State<'_, AgentContext>,
    unit_of_work_factory: tauri::State<'_, Arc<dyn UnitOfWorkFactory>>,
    static_chat_repo: tauri::State<'_, Arc<dyn ChatRepo>>,
) -> Result<(), AppError> {
    let unit_of_work = unit_of_work_factory.create().await?;
    let (user_chat_msg, mut stream) = {
        let agent_repo = unit_of_work.agent_repo();
        let current_agent = agent_repo
            .get_current_agent()
            .await?
            .ok_or_else(|| AppError::AgentRequired)?;
        let agent = Agent::from(current_agent);
        let chat_repo = unit_of_work.chat_repo();
        let user_chat_msg = chat_repo
            .create_chat_message(CreateChatMessage {
                id: Uuid::new_v4(),
                chat_id,
                role: "user".into(),
                content: content.clone(),
                status: ChatMessageStatus::Completed,
            })
            .await?;
        let _ = app_handle
            .emit("chat_message_created", &user_chat_msg)
            .inspect_err(|e| {
                log::error!("failed to emit response chunk: {e}");
            });

        let mut config = agent
            .create_text_gen_params(agent_context.inner().clone(), chat_id)
            .await?
            .ok_or_else(|| AppError::AgentTextGenParamsRequired)?;
        config.push_message_str(&content);
        let stream = agent
            .generate_text(agent_context.inner().clone(), config)
            .await?;
        (user_chat_msg, stream)
    };

    let model_chat_msg = {
        let chat_repo = unit_of_work.chat_repo();
        chat_repo
            .create_chat_message(CreateChatMessage {
                id: Uuid::new_v4(),
                chat_id,
                role: "model".into(),
                content: String::new(),
                status: ChatMessageStatus::Pending,
            })
            .await?
    };
    let _ = app_handle
        .emit("chat_message_created", model_chat_msg.clone())
        .inspect_err(|e| {
            log::error!("failed to emit response chunk: {e}");
        });

    unit_of_work.commit().await.inspect_err(|_| {
        let _ = app_handle.emit(
            "chat_message_rollback",
            ChatMessageRollbackPayload {
                chat_id: chat_id,
                message_id: user_chat_msg.id,
            },
        );
        let _ = app_handle.emit(
            "chat_message_rollback",
            ChatMessageRollbackPayload {
                chat_id: chat_id,
                message_id: model_chat_msg.id,
            },
        );
    })?;
    let chat_repo = static_chat_repo.inner().clone();
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
                        let _ = chat_repo
                            .update_chat_message(
                                model_chat_msg.id,
                                UpdateChatMessage {
                                    content: Some(text.clone()),
                                    ..Default::default()
                                },
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
                    let _ = chat_repo
                        .update_chat_message(
                            model_chat_msg.id,
                            UpdateChatMessage {
                                content: Some(text.clone()),
                                status: Some(ChatMessageStatus::Failed),
                                ..Default::default()
                            },
                        )
                        .await
                        .inspect_err(|e| {
                            log::error!("failed to update chat message status and content: {e}");
                        });
                    return;
                }
            }
        }
        let _ = chat_repo
            .update_chat_message(
                model_chat_msg.id,
                UpdateChatMessage {
                    content: match chunk_count {
                        0 => None,
                        _ => Some(text.clone()),
                    },
                    status: Some(ChatMessageStatus::Completed),
                    ..Default::default()
                },
            )
            .await
            .inspect_err(|e| {
                log::error!("failed to update chat message status and content: {e}");
            });
        let _ = app_handle
            .emit(
                "chat_message_status_changed",
                ChatMessageStatusChangedPayload {
                    chat_id,
                    message_id: model_chat_msg.id,
                    status: ChatMessageStatus::Completed,
                },
            )
            .inspect_err(|e| {
                log::error!("failed to emit chat_message_completed: {e}");
            });
    });

    Ok(())
}

#[tauri::command]
pub async fn get_chat(
    id: Uuid,
    chat_repo: tauri::State<'_, Arc<dyn ChatRepo>>,
) -> Result<Option<ChatRow>, AppError> {
    chat_repo.get_chat(id).await
}

#[tauri::command]
pub async fn get_chat_messages(
    id: Uuid,
    chat_repo: tauri::State<'_, Arc<dyn ChatRepo>>,
) -> Result<Vec<ChatMessageRow>, AppError> {
    chat_repo.get_chat_messages(id).await
}
