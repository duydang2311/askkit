pub mod cmds;
pub mod google;
pub mod repo;

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    agent::{
        google::{GoogleAgent, GoogleTextGenParams, GoogleTextGenParamsMessage},
        repo::AgentRepo,
    },
    chat::repo::ChatRepo,
    cipher::Cipher,
    common::{entity::agent::AgentRow, error::AppError, http::HttpClientManager},
};

#[derive(Clone, sqlx::Type, Serialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentProvider {
    Google,
    OpenAI,
}

pub enum Agent {
    Gemini(GoogleAgent),
}

pub enum AgentTextGenParams {
    Google(GoogleTextGenParams),
}

pub enum AgentTextGenParamsMessage {
    Google(GoogleTextGenParamsMessage),
}

pub trait AgentTextGenParamsApi {
    type Message;

    fn push_message(&mut self, message: Self::Message);
    fn push_message_str(&mut self, message: &str);
}

#[async_trait]
pub trait AgentApi {
    type TextGenParams;

    async fn generate_text(
        self,
        context: AgentContext,
        params: Self::TextGenParams,
    ) -> Result<impl Stream<Item = Result<AgentTextGenResult, AppError>>, AppError>;

    async fn create_text_gen_params(
        &self,
        context: AgentContext,
        chat_id: Uuid,
    ) -> Result<Option<Self::TextGenParams>, AppError>;
}

#[derive(Serialize, Clone, Debug)]
pub struct AgentTextGenResult {
    pub text: String,
}

pub struct AgentContext {
    pub http_client_manager: Arc<HttpClientManager>,
    pub agent_repo: Arc<dyn AgentRepo>,
    pub chat_repo: Arc<dyn ChatRepo>,
    pub cipher: Arc<dyn Cipher>,
}

impl AgentContext {
    pub fn new(
        http_client_manager: Arc<HttpClientManager>,
        agent_repo: Arc<dyn AgentRepo>,
        chat_repo: Arc<dyn ChatRepo>,
        cipher: Arc<dyn Cipher>,
    ) -> Self {
        Self {
            http_client_manager,
            agent_repo,
            chat_repo,
            cipher,
        }
    }
}

impl Clone for AgentContext {
    fn clone(&self) -> Self {
        Self {
            http_client_manager: Arc::clone(&self.http_client_manager),
            agent_repo: Arc::clone(&self.agent_repo),
            chat_repo: Arc::clone(&self.chat_repo),
            cipher: Arc::clone(&self.cipher),
        }
    }
}

impl From<AgentRow> for Agent {
    fn from(row: AgentRow) -> Self {
        match row.provider {
            AgentProvider::Google => Self::Gemini(GoogleAgent {
                id: row.id,
                model: row.model,
            }),
            AgentProvider::OpenAI => todo!("Implement OpenAI agent"),
        }
    }
}

#[async_trait]
impl AgentApi for Agent {
    type TextGenParams = AgentTextGenParams;

    async fn generate_text(
        self,
        context: AgentContext,
        params: Self::TextGenParams,
    ) -> Result<impl Stream<Item = Result<AgentTextGenResult, AppError>>, AppError> {
        match (self, params) {
            (Agent::Gemini(agent), AgentTextGenParams::Google(params)) => {
                agent.generate_text(context, params).await
            }
        }
    }

    async fn create_text_gen_params(
        &self,
        context: AgentContext,
        chat_id: Uuid,
    ) -> Result<Option<Self::TextGenParams>, AppError> {
        match self {
            Agent::Gemini(agent) => agent
                .create_text_gen_params(context, chat_id)
                .await
                .map(|a| a.map(|a| AgentTextGenParams::Google(a))),
        }
    }
}

impl AgentTextGenParamsApi for AgentTextGenParams {
    type Message = AgentTextGenParamsMessage;

    fn push_message(&mut self, message: Self::Message) {
        match (self, message) {
            (AgentTextGenParams::Google(params), AgentTextGenParamsMessage::Google(message)) => {
                params.push_message(message);
            }
        }
    }

    fn push_message_str(&mut self, message: &str) {
        match self {
            AgentTextGenParams::Google(params) => {
                params.push_message_str(message);
            }
        }
    }
}
