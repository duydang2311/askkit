use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ChatRow {
    pub created_at: i64,
    pub id: Uuid,
    pub title: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageRow {
    pub created_at: i64,
    pub id: Uuid,
    pub chat_id: Uuid,
    pub role: String,
    pub content: String,
    pub status: ChatMessageStatus,
}

#[derive(Debug, Serialize, Clone, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ChatMessageStatus {
    Pending,
    Completed,
    Failed,
}
