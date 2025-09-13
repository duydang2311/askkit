use serde::Serialize;
use uuid::Uuid;

use crate::agent::AgentProvider;

#[derive(sqlx::FromRow, Serialize)]
pub struct AgentRow {
    pub created_at: i64,
    pub updated_at: i64,
    pub id: Uuid,
    pub provider: AgentProvider,
    pub model: String,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct AgentConfigRow {
    pub created_at: i64,
    pub updated_at: i64,
    pub agent_id: Uuid,
    pub api_key: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct AgentProviderRow {
    pub created_at: i64,
    pub updated_at: i64,
    pub id: Uuid,
    pub provider: AgentProvider,
    pub api_key: Option<String>,
}
