use async_trait::async_trait;
use uuid::Uuid;

use crate::common::{
    entity::chat::{ChatMessageRow, ChatMessageStatus},
    error::AppError,
};

pub mod sqlite;

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

#[async_trait]
pub trait ChatRepo: Send + Sync {
    async fn get_chat_messages(&self, chat_id: Uuid) -> Result<Vec<ChatMessageRow>, AppError>;
    async fn create_chat_message(
        &self,
        message: CreateChatMessage,
    ) -> Result<ChatMessageRow, AppError>;
    async fn update_chat_message(
        &self,
        id: Uuid,
        update: UpdateChatMessage,
    ) -> Result<(), AppError>;
}
