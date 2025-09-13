use async_trait::async_trait;
use sqlx::{Executor, Pool, QueryBuilder, Sqlite, SqliteTransaction};
use std::sync::Arc;
use tauri::async_runtime::Mutex;
use uuid::Uuid;

use crate::{
    chat::repo::{ChatRepo, CreateChatMessage, UpdateChatMessage},
    common::{
        entity::chat::{ChatMessageRow, ChatRow},
        error::AppError,
    },
};

pub struct SqliteChatRepo {
    db_pool: Arc<Pool<Sqlite>>,
}

pub struct TransactionalSqliteChatRepo<'a> {
    tx: Arc<Mutex<SqliteTransaction<'a>>>,
}

impl SqliteChatRepo {
    pub fn new(db_pool: Arc<Pool<Sqlite>>) -> Self {
        Self { db_pool }
    }
}

impl<'a> TransactionalSqliteChatRepo<'a> {
    pub fn new(tx: Arc<Mutex<SqliteTransaction<'a>>>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl ChatRepo for SqliteChatRepo {
    async fn get_chat_messages(&self, chat_id: Uuid) -> Result<Vec<ChatMessageRow>, AppError> {
        get_chat_messages(&*self.db_pool, chat_id).await
    }

    async fn create_chat_message(
        &self,
        message: CreateChatMessage,
    ) -> Result<ChatMessageRow, AppError> {
        create_chat_message(&*self.db_pool, message).await
    }

    async fn update_chat_message(
        &self,
        id: Uuid,
        update: UpdateChatMessage,
    ) -> Result<(), AppError> {
        update_chat_message(&*self.db_pool, id, update).await
    }

    async fn get_chat(&self, id: Uuid) -> Result<Option<ChatRow>, AppError> {
        get_chat(&*self.db_pool, id).await
    }
}

#[async_trait]
impl<'a> ChatRepo for TransactionalSqliteChatRepo<'a> {
    async fn get_chat_messages(&self, chat_id: Uuid) -> Result<Vec<ChatMessageRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_chat_messages(&mut **tx, chat_id).await
    }

    async fn create_chat_message(
        &self,
        message: CreateChatMessage,
    ) -> Result<ChatMessageRow, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        create_chat_message(&mut **tx, message).await
    }

    async fn update_chat_message(
        &self,
        id: Uuid,
        update: UpdateChatMessage,
    ) -> Result<(), AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        update_chat_message(&mut **tx, id, update).await
    }

    async fn get_chat(&self, id: Uuid) -> Result<Option<ChatRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_chat(&mut **tx, id).await
    }
}

async fn get_chat_messages<'a, E>(
    executor: E,
    chat_id: Uuid,
) -> Result<Vec<ChatMessageRow>, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query_as::<_, ChatMessageRow>(
        "select * from chat_messages where chat_id = ?1 order by created_at asc",
    )
    .bind(&chat_id)
    .fetch_all(executor)
    .await
    .map_err(AppError::from)
}

async fn create_chat_message<'a, E>(
    executor: E,
    message: CreateChatMessage,
) -> Result<ChatMessageRow, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let created_at = sqlx::query_scalar::<_, i64>("insert into chat_messages (id, chat_id, role, content, status) values (?1, ?2, ?3, ?4, ?5) returning created_at")
            .bind(message.id)
            .bind(message.chat_id)
            .bind(&message.role)
            .bind(&message.content)
            .bind(&message.status)
            .fetch_one(executor)
            .await
            .map_err(AppError::from)?;

    Ok(ChatMessageRow {
        created_at,
        id: message.id,
        chat_id: message.chat_id,
        role: message.role,
        content: message.content,
        status: message.status,
    })
}

async fn update_chat_message<'a, E>(
    executor: E,
    id: Uuid,
    update: UpdateChatMessage,
) -> Result<(), AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let mut qb = QueryBuilder::new("update chat_messages set ");
    let mut sep = qb.separated(", ");
    if let Some(role) = update.role {
        sep.push("role = ").push_bind_unseparated(role);
    }
    if let Some(content) = update.content {
        sep.push("content = ").push_bind_unseparated(content);
    }
    if let Some(status) = update.status {
        sep.push("status = ").push_bind_unseparated(status);
    }
    qb.push(" where id = ").push_bind(id);

    qb.build().execute(executor).await.map_err(AppError::from)?;
    Ok(())
}

async fn get_chat<'a, E>(executor: E, id: Uuid) -> Result<Option<ChatRow>, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query_as::<_, ChatRow>("select * from chats where id = ?1")
        .bind(&id)
        .fetch_optional(executor)
        .await
        .map_err(AppError::from)
}
