use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqliteTransaction};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    agent::repo::{sqlite::TransactionalSqliteAgentRepo, AgentRepo},
    chat::repo::{sqlite::TransactionalSqliteChatRepo, ChatRepo},
    cipher::Cipher,
    common::error::AppError,
};

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    fn chat_repo(&self) -> Box<dyn ChatRepo>;
    fn agent_repo(&self) -> Box<dyn AgentRepo>;
    async fn commit(self: Box<Self>) -> Result<(), AppError>;
}

#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, AppError>;
}

pub struct SqliteUnitOfWork {
    tx: Arc<Mutex<SqliteTransaction<'static>>>,
    cipher: Arc<dyn Cipher>,
}

pub struct SqliteUnitOfWorkFactory {
    db_pool: Arc<Pool<Sqlite>>,
    cipher: Arc<dyn Cipher>,
}

impl SqliteUnitOfWork {
    pub fn new(tx: SqliteTransaction<'static>, cipher: Arc<dyn Cipher>) -> Self {
        Self {
            tx: Arc::new(Mutex::new(tx)),
            cipher,
        }
    }
}

impl SqliteUnitOfWorkFactory {
    pub fn new(db_pool: Arc<Pool<Sqlite>>, cipher: Arc<dyn Cipher>) -> Self {
        Self { db_pool, cipher }
    }
}

#[async_trait]
impl UnitOfWorkFactory for SqliteUnitOfWorkFactory {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, AppError> {
        let tx = self.db_pool.begin().await.map_err(AppError::from)?;
        Ok(Box::new(SqliteUnitOfWork::new(tx, self.cipher.clone())))
    }
}

#[async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    fn chat_repo(&self) -> Box<dyn ChatRepo> {
        Box::new(TransactionalSqliteChatRepo::new(self.tx.clone()))
    }

    fn agent_repo(&self) -> Box<dyn AgentRepo> {
        Box::new(TransactionalSqliteAgentRepo::new(
            self.tx.clone(),
            self.cipher.clone(),
        ))
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        let mutex = match Arc::try_unwrap(self.tx) {
            Ok(mutex) => mutex,
            Err(_) => {
                return Err(AppError::TransactionInUse);
            }
        };

        mutex.into_inner().commit().await.map_err(AppError::from)
    }
}
