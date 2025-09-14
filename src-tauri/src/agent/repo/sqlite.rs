use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Executor, Pool, Sqlite, SqliteExecutor, SqliteTransaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    agent::repo::{
        AgentRepo, CreateAgent, CreateAgentConfig, CreateAgentProvider, UpdateAgent,
        UpdateAgentConfig, UpdateAgentProvider, UpdateCurrentAgent, UpsertAgentConfig,
    },
    common::{
        entity::agent::{AgentConfigRow, AgentProviderRow, AgentRow},
        error::AppError,
    },
};

pub struct SqliteAgentRepo {
    db_pool: Arc<Pool<Sqlite>>,
}

pub struct TransactionalSqliteAgentRepo<'a> {
    tx: Arc<Mutex<SqliteTransaction<'a>>>,
}

impl SqliteAgentRepo {
    pub fn new(db_pool: Arc<Pool<Sqlite>>) -> Self {
        Self { db_pool }
    }
}

impl<'a> TransactionalSqliteAgentRepo<'a> {
    pub fn new(tx: Arc<Mutex<SqliteTransaction<'a>>>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl AgentRepo for SqliteAgentRepo {
    async fn get_agents(&self) -> Result<Vec<AgentRow>, AppError> {
        get_agents(&*self.db_pool).await
    }

    async fn get_agent(&self, agent_id: Uuid) -> Result<Option<AgentRow>, AppError> {
        get_agent(&*self.db_pool, agent_id).await
    }

    async fn get_current_agent(&self) -> Result<Option<AgentRow>, AppError> {
        get_current_agent(&*self.db_pool).await
    }

    async fn create_agent(&self, create: CreateAgent) -> Result<AgentRow, AppError> {
        create_agent(&*self.db_pool, create).await
    }

    async fn update_agent(&self, id: String, update: UpdateAgent) -> Result<(), AppError> {
        update_agent(&*self.db_pool, id, update).await
    }

    async fn create_provider(
        &self,
        create: CreateAgentProvider,
    ) -> Result<AgentProviderRow, AppError> {
        create_provider(&*self.db_pool, create).await
    }

    async fn update_provider(
        &self,
        id: String,
        update: UpdateAgentProvider,
    ) -> Result<(), AppError> {
        update_provider(&*self.db_pool, id, update).await
    }

    async fn create_agent_config(
        &self,
        create: CreateAgentConfig,
    ) -> Result<AgentConfigRow, AppError> {
        create_agent_config(&*self.db_pool, create).await
    }

    async fn get_agent_config(&self, agent_id: Uuid) -> Result<Option<AgentConfigRow>, AppError> {
        get_agent_config(&*self.db_pool, agent_id).await
    }

    async fn update_current_agent(&self, update: UpdateCurrentAgent) -> Result<(), AppError> {
        update_current_agent(&*self.db_pool, update).await
    }

    async fn update_agent_config(
        &self,
        agent_id: Uuid,
        update: UpdateAgentConfig,
    ) -> Result<u64, AppError> {
        update_agent_config(&*self.db_pool, agent_id, update).await
    }

    async fn upsert_agent_config(
        &self,
        agent_id: Uuid,
        upsert: UpsertAgentConfig,
    ) -> Result<u64, AppError> {
        upsert_agent_config(&*self.db_pool, agent_id, upsert).await
    }
}

#[async_trait]
impl<'a> AgentRepo for TransactionalSqliteAgentRepo<'a> {
    async fn get_agents(&self) -> Result<Vec<AgentRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_agents(&mut **tx).await
    }

    async fn get_agent(&self, agent_id: Uuid) -> Result<Option<AgentRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_agent(&mut **tx, agent_id).await
    }

    async fn get_current_agent(&self) -> Result<Option<AgentRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_current_agent(&mut **tx).await
    }

    async fn create_agent(&self, create: CreateAgent) -> Result<AgentRow, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        create_agent(&mut **tx, create).await
    }

    async fn update_agent(&self, id: String, update: UpdateAgent) -> Result<(), AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        update_agent(&mut **tx, id, update).await
    }

    async fn create_provider(
        &self,
        create: CreateAgentProvider,
    ) -> Result<AgentProviderRow, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        create_provider(&mut **tx, create).await
    }

    async fn update_provider(
        &self,
        id: String,
        update: UpdateAgentProvider,
    ) -> Result<(), AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        update_provider(&mut **tx, id, update).await
    }

    async fn create_agent_config(
        &self,
        create: CreateAgentConfig,
    ) -> Result<AgentConfigRow, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        create_agent_config(&mut **tx, create).await
    }

    async fn get_agent_config(&self, agent_id: Uuid) -> Result<Option<AgentConfigRow>, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        get_agent_config(&mut **tx, agent_id).await
    }

    async fn update_current_agent(&self, update: UpdateCurrentAgent) -> Result<(), AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        update_current_agent(&mut **tx, update).await
    }

    async fn update_agent_config(
        &self,
        agent_id: Uuid,
        update: UpdateAgentConfig,
    ) -> Result<u64, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        update_agent_config(&mut **tx, agent_id, update).await
    }

    async fn upsert_agent_config(
        &self,
        agent_id: Uuid,
        upsert: UpsertAgentConfig,
    ) -> Result<u64, AppError> {
        let mut tx = self.tx.try_lock().map_err(AppError::from)?;
        upsert_agent_config(&mut **tx, agent_id, upsert).await
    }
}

async fn get_agents<'a, E>(executor: E) -> Result<Vec<AgentRow>, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query_as::<_, AgentRow>("select * from agents")
        .fetch_all(executor)
        .await
        .map_err(AppError::from)
}

async fn get_agent<'a, E>(executor: E, agent_id: Uuid) -> Result<Option<AgentRow>, AppError>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_as::<_, AgentRow>("select * from agents where id = ?1")
        .bind(agent_id)
        .fetch_optional(executor)
        .await
        .map_err(AppError::from)
}

async fn get_current_agent<'a, E>(executor: E) -> Result<Option<AgentRow>, AppError>
where
    E: SqliteExecutor<'a>,
{
    sqlx::query_as::<_, AgentRow>("select agents.* from agents inner join current_agent on current_agent.agent_id = agents.id")
        .fetch_optional(executor)
        .await
        .map_err(AppError::from)
}

async fn create_agent<'a, E>(executor: E, create: CreateAgent) -> Result<AgentRow, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let (created_at, updated_at): (i64, i64) = sqlx::query_as("insert into agents (id, provider, model) values (?1, ?2, ?3) returning created_at, updated_at")
        .bind(&create.id)
        .bind(&create.provider)
        .bind(&create.model)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)?;
    Ok(AgentRow {
        created_at,
        updated_at,
        id: create.id,
        provider: create.provider,
        model: create.model,
    })
}

async fn update_agent<'a, E>(executor: E, id: String, update: UpdateAgent) -> Result<(), AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let mut qb = sqlx::QueryBuilder::new("update agents set ");
    let mut separated = qb.separated(", ");
    if let Some(provider) = update.provider {
        separated.push("provider = ").push_bind(provider);
    }
    if let Some(model) = update.model {
        separated.push("model = ").push_bind(model);
    }
    qb.push(" where id = ").push_bind(&id);
    qb.build().execute(executor).await.map_err(AppError::from)?;
    Ok(())
}

async fn create_provider<'a, E>(
    executor: E,
    create: CreateAgentProvider,
) -> Result<AgentProviderRow, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let (created_at, updated_at): (i64, i64) = sqlx::query_as("insert into agent_providers (id, provider, api_key) values (?1, ?2, $3) returning created_at, updated_at")
        .bind(&create.id)
        .bind(&create.provider)
        .bind(&create.api_key)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)?;
    Ok(AgentProviderRow {
        id: create.id,
        created_at,
        updated_at,
        provider: create.provider,
        api_key: create.api_key,
    })
}

async fn update_provider<'a, E>(
    executor: E,
    id: String,
    update: UpdateAgentProvider,
) -> Result<(), AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let mut qb = sqlx::QueryBuilder::new("update agent_providers set ");
    let mut separated = qb.separated(", ");
    if let Some(api_key) = update.api_key {
        separated.push("api_key = ").push_bind(api_key);
    }
    qb.push(" where id = ").push_bind(&id);
    qb.build().execute(executor).await.map_err(AppError::from)?;
    Ok(())
}

async fn create_agent_config<'a, E>(
    executor: E,
    create: CreateAgentConfig,
) -> Result<AgentConfigRow, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let (created_at, updated_at): (i64, i64) = sqlx::query_as("insert into agent_configs (agent_id, api_key) values (?1, ?2) returning created_at, updated_at")
        .bind(&create.agent_id)
        .bind(&create.api_key)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)?;
    Ok(AgentConfigRow {
        created_at,
        updated_at,
        agent_id: create.agent_id,
        api_key: create.api_key,
    })
}

async fn get_agent_config<'a, E>(
    executor: E,
    agent_id: Uuid,
) -> Result<Option<AgentConfigRow>, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query_as::<_, AgentConfigRow>("select * from agent_configs where agent_id = ?1")
        .bind(agent_id)
        .fetch_optional(executor)
        .await
        .map_err(AppError::from)
}

async fn update_current_agent<'a, E>(
    executor: E,
    update: UpdateCurrentAgent,
) -> Result<(), AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query("insert into current_agent (id, agent_id) values (1, ?1) on conflict(id) do update set agent_id = excluded.agent_id")
        .bind(update.agent_id)
        .execute(executor)
        .await
        .map_err(AppError::from)?;
    Ok(())
}

async fn update_agent_config<'a, E>(
    executor: E,
    agent_id: Uuid,
    update: UpdateAgentConfig,
) -> Result<u64, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let mut qb = sqlx::QueryBuilder::new("update agent_configs set ");
    let mut separated = qb.separated(", ");
    if let Some(api_key) = update.api_key {
        separated.push("api_key = ").push_bind_unseparated(api_key);
    }
    qb.push(" where agent_id = ").push_bind(&agent_id);
    let result = qb.build().execute(executor).await.map_err(AppError::from)?;
    Ok(result.rows_affected())
}

async fn upsert_agent_config<'a, E>(
    executor: E,
    agent_id: Uuid,
    update: UpsertAgentConfig,
) -> Result<u64, AppError>
where
    E: Executor<'a, Database = Sqlite>,
{
    let mut qb = sqlx::QueryBuilder::new("insert into agent_configs (");
    {
        let mut fields = qb.separated(", ");
        fields.push("agent_id");
        if let Some(_) = update.api_key {
            fields.push("api_key");
        }
    }

    {
        qb.push(") values (");
        let mut values = qb.separated(", ");
        values.push_bind(&agent_id);
        if let Some(api_key) = &update.api_key {
            values.push_bind(api_key);
        }
    }

    {
        qb.push(") on conflict (agent_id) do update set ");
        let mut updates = qb.separated(", ");
        if let Some(_) = &update.api_key {
            updates.push("api_key = excluded.api_key");
        }
    }
    let result = qb.build().execute(executor).await.map_err(AppError::from)?;
    Ok(result.rows_affected())
}
