use serde::Deserialize;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::{
    agent::repo::{AgentRepo, UpdateCurrentAgent, UpsertAgentConfig},
    cipher::Cipher,
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
    cipher: State<'_, Arc<dyn Cipher>>,
) -> Result<u64, AppError> {
    agent_repo
        .upsert_agent_config(
            id,
            UpsertAgentConfig {
                api_key: upsert
                    .api_key
                    .map(|a| {
                        let trimmed = a.trim();
                        if trimmed.is_empty() {
                            Ok(String::new())
                        } else {
                            cipher.encrypt_str_base64(trimmed)
                        }
                    })
                    .transpose()?,
            },
        )
        .await
}

#[tauri::command]
pub async fn decrypt_agent_ciphertext(
    ciphertext: String,
    cipher: State<'_, Arc<dyn Cipher>>,
) -> Result<String, AppError> {
    cipher.decrypt_base64_str(&ciphertext)
}
