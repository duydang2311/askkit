use std::sync::Arc;

use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::{
    agent::repo::{AgentRepo, UpdateCurrentAgent, UpsertAgentConfig},
    common::{
        entity::agent::{AgentConfigRow, AgentRow},
        error::AppError,
    },
};

#[derive(Deserialize)]
pub struct UpsertAgentConfigCmd {
    pub api_key: Option<String>,
}

#[tauri::command]
pub async fn get_agents(
    agent_repo: State<'_, Arc<dyn AgentRepo>>,
) -> Result<Vec<AgentRow>, AppError> {
    agent_repo.get_agents().await
}

#[tauri::command]
pub async fn get_current_agent(
    agent_repo: State<'_, Arc<dyn AgentRepo>>,
) -> Result<Option<AgentRow>, AppError> {
    agent_repo.get_current_agent().await
}

#[tauri::command]
pub async fn update_current_agent(
    agent_id: Uuid,
    agent_repo: State<'_, Arc<dyn AgentRepo>>,
) -> Result<(), AppError> {
    agent_repo
        .update_current_agent(UpdateCurrentAgent { agent_id })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_agent_config(
    id: Uuid,
    agent_repo: State<'_, Arc<dyn AgentRepo>>,
) -> Result<Option<AgentConfigRow>, AppError> {
    agent_repo.get_agent_config(id).await
}

#[tauri::command]
pub async fn upsert_agent_config(
    id: Uuid,
    upsert: UpsertAgentConfigCmd,
    agent_repo: State<'_, Arc<dyn AgentRepo>>,
) -> Result<u64, AppError> {
    agent_repo
        .upsert_agent_config(
            id,
            UpsertAgentConfig {
                api_key: upsert.api_key,
            },
        )
        .await
}
