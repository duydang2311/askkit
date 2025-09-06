use sqlx::{Pool, QueryBuilder, Sqlite};
use uuid::Uuid;

use crate::common::{entity::chat::{ChatMessage, ChatMessageStatus}, errors::AppError};

pub struct CreateChatMessage {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: String,
    pub content: String,
    pub status: ChatMessageStatus,
}

#[derive(Default)]
pub struct UpdateChatMessage {
    pub role: Option<String>,
    pub content: Option<String>,
    pub status: Option<ChatMessageStatus>,
}

pub async fn create_chat_message(
    chat_message: CreateChatMessage,
    db_pool: &Pool<Sqlite>,
) -> Result<ChatMessage, AppError> {
    let created_at: i64 = sqlx::query_scalar("insert into chat_messages (id, chat_id, role, content, status) values (?1, ?2, ?3, ?4, ?5) returning created_at")
        .bind(&chat_message.id)
        .bind(&chat_message.chat_id)
        .bind(&chat_message.role)
        .bind(&chat_message.content)
        .bind(&chat_message.status)
        .fetch_one(db_pool)
        .await
        .map_err(|e| AppError::Unknown(Some(Box::new(e))))?;

    Ok(ChatMessage {
        created_at,
        id: chat_message.id,
        chat_id: chat_message.chat_id,
        role: chat_message.role,
        content: chat_message.content,
        status: chat_message.status,
    })
}

pub async fn update_chat_message(
    id: Uuid,
    update: UpdateChatMessage,
    db_pool: &Pool<Sqlite>,
) -> Result<(), AppError> {
    let mut builder = QueryBuilder::new("update chat_messages set ");
    let mut sep = builder.separated(", ");
    if let Some(role) = update.role {
        sep.push("role = ").push_bind_unseparated(role);
    }
    if let Some(content) = update.content {
        sep.push("content = ").push_bind_unseparated(content);
    }
    if let Some(status) = update.status {
        sep.push("status = ").push_bind_unseparated(status);
    }
    builder.push(" where id = ").push_bind(id);
    builder
        .build()
        .execute(db_pool)
        .await
        .map_err(|e| AppError::Unknown(Some(Box::new(e))))?;
    Ok(())
}

pub async fn get_chat_messages(
    chat_id: Uuid,
    db_pool: &Pool<Sqlite>,
) -> Result<Vec<ChatMessage>, AppError> {
    let messages = sqlx::query_as::<_, (i64, Uuid, String, String, ChatMessageStatus)>(
        "select created_at, id, role, content, status from chat_messages where chat_id = ?1",
    )
    .bind(chat_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| AppError::Unknown(Some(Box::new(e))))?;
    Ok(messages
        .into_iter()
        .map(|(created_at, id, role, content, status)| ChatMessage {
            created_at,
            id,
            chat_id,
            role,
            content,
            status,
        })
        .collect())
}
