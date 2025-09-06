use std::sync::Arc;

use futures_util::Stream;
use serde::Serialize;
use sqlx::{Database, Pool};

use crate::common::{
    agent_gemini::{GeminiAgent, GeminiTextGenRequestParams},
    errors::AppError,
    http::HttpClientManager,
};

pub enum Agent {
    Gemini(GeminiAgent),
}

pub trait AgentApi {
    type TextGenParams;

    async fn generate_text<DB: Database>(
        self,
        context: AgentContext<DB>,
        params: Self::TextGenParams,
    ) -> Result<impl Stream<Item = Result<AgentTextGenResult, AppError>>, AppError>;
}

#[derive(Serialize, Clone, Debug)]
pub struct AgentTextGenResult {
    pub text: String,
}

pub enum TextGenRequestParams {
    Gemini(GeminiTextGenRequestParams),
}

pub struct AgentContext<DB: Database> {
    pub http_client_manager: Arc<HttpClientManager>,
    pub db_pool: Arc<Pool<DB>>,
}

impl<DB: Database> AgentContext<DB> {
    pub fn new(http_client_manager: Arc<HttpClientManager>, db_pool: Arc<Pool<DB>>) -> Self {
        Self {
            http_client_manager,
            db_pool,
        }
    }
}

impl<DB: Database> Clone for AgentContext<DB> {
    fn clone(&self) -> Self {
        Self {
            http_client_manager: Arc::clone(&self.http_client_manager),
            db_pool: Arc::clone(&self.db_pool),
        }
    }
}
