use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    agent::AgentProvider,
    common::{
        entity::agent::{AgentConfigRow, AgentProviderRow, AgentRow},
        error::AppError,
    },
};

pub mod sqlite;

pub struct CreateAgent {
    pub id: Uuid,
    pub provider: AgentProvider,
    pub model: String,
}

pub struct CreateAgentProvider {
    pub id: Uuid,
    pub provider: AgentProvider,
    pub api_key: Option<String>,
}

pub struct UpdateAgent {
    pub id: Uuid,
    pub provider: Option<AgentProvider>,
    pub model: Option<String>,
}

pub struct UpdateAgentProvider {
    pub api_key: Option<String>,
}

pub struct CreateAgentConfig {
    pub agent_id: Uuid,
    pub api_key: Option<String>,
}

pub struct UpdateCurrentAgent {
    pub agent_id: Uuid,
}

pub struct UpdateAgentConfig {
    pub api_key: Option<String>,
}

pub struct UpsertAgentConfig {
    pub api_key: Option<String>,
}

#[async_trait]
pub trait AgentRepo: Send + Sync {
    async fn get_agents(&self) -> Result<Vec<AgentRow>, AppError>;
    async fn create_agent(&self, create: CreateAgent) -> Result<AgentRow, AppError>;
    async fn get_agent(&self, agent_id: Uuid) -> Result<Option<AgentRow>, AppError>;
    async fn update_agent(&self, id: String, update: UpdateAgent) -> Result<(), AppError>;
    async fn get_current_agent(&self) -> Result<Option<AgentRow>, AppError>;
    async fn update_current_agent(&self, update: UpdateCurrentAgent) -> Result<(), AppError>;
    async fn create_provider(
        &self,
        create: CreateAgentProvider,
    ) -> Result<AgentProviderRow, AppError>;
    async fn update_provider(
        &self,
        id: String,
        update: UpdateAgentProvider,
    ) -> Result<(), AppError>;
    async fn create_agent_config(
        &self,
        create: CreateAgentConfig,
    ) -> Result<AgentConfigRow, AppError>;
    async fn get_agent_config(&self, agent_id: Uuid) -> Result<Option<AgentConfigRow>, AppError>;
    async fn update_agent_config(
        &self,
        agent_id: Uuid,
        update: UpdateAgentConfig,
    ) -> Result<u64, AppError>;
    async fn upsert_agent_config(
        &self,
        agent_id: Uuid,
        update: UpsertAgentConfig,
    ) -> Result<u64, AppError>;
}
