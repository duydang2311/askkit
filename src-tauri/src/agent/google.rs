use std::pin::Pin;

use async_trait::async_trait;
use futures_util::{self, stream, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::{AgentApi, AgentContext, AgentTextGenParamsApi, AgentTextGenResult};
use crate::common::error::AppError;

const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_X_GOOG_API_KEY: &str = "X-goog-api-key";

#[derive(Clone)]
pub struct GoogleAgent {
    pub id: Uuid,
    pub model: String,
}

pub struct GoogleTextGenParams {
    pub api_key: String,
    pub messages: Vec<GoogleTextGenParamsMessage>,
}

pub struct GoogleTextGenParamsMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct GoogleTextGenRequestBody {
    pub contents: Vec<GoogleTextGenRequestBodyContent>,
}

#[derive(Serialize)]
pub struct GoogleTextGenRequestBodyContent {
    pub role: String,
    pub parts: Vec<GoogleTextGenRequestBodyContentPart>,
}

#[derive(Serialize)]
pub struct GoogleTextGenRequestBodyContentPart {
    pub text: String,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBody {
    candidates: Vec<GeminiTextGenResponseBodyCandidate>,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBodyCandidate {
    pub content: GoogleTextGenResponseBodyCandidateContent,
}

#[derive(Deserialize)]
pub struct GoogleTextGenResponseBodyCandidateContent {
    pub parts: Vec<GoogleTextGenResponseBodyCandidateContentPart>,
}

#[derive(Deserialize)]
pub struct GoogleTextGenResponseBodyCandidateContentPart {
    pub text: String,
}

#[async_trait]
impl AgentApi for GoogleAgent {
    type TextGenParams = GoogleTextGenParams;

    async fn generate_text(
        self,
        context: AgentContext,
        params: Self::TextGenParams,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentTextGenResult, AppError>> + Send>>, AppError> {
        let client = context.http_client_manager.get_client();
        let body = GoogleTextGenRequestBody {
            contents: params
                .messages
                .into_iter()
                .map(|a| GoogleTextGenRequestBodyContent {
                    role: a.role,
                    parts: vec![GoogleTextGenRequestBodyContentPart { text: a.content }],
                })
                .collect(),
        };
        let stream = client
            .request(
                reqwest::Method::POST,
                format!("https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse", &self.model),
            )
            .header(HEADER_CONTENT_TYPE, "application/json")
            .header(HEADER_X_GOOG_API_KEY, params.api_key)
            .json(&body)
            .send()
            .await
            .map_err(AppError::from)
            ?.bytes_stream()
            .map_err(AppError::from)
            .map_ok(|bytes| {
                let results: Vec<Result<AgentTextGenResult, AppError>> = match std::str::from_utf8(&bytes) {
                    Ok(text) => {
                        text.lines()
                            .filter_map(|line| line.strip_prefix("data:").map(str::trim))
                            .filter(|line| !line.is_empty())
                            .flat_map(|json| {
                                serde_json::from_str::<GeminiTextGenResponseBody>(json)
                                    .map(|body| {
                                        body.candidates
                                            .into_iter()
                                            .flat_map(|c| c.content.parts)
                                            .map(|p| Ok(AgentTextGenResult { text: p.text }))
                                            .collect()
                                    })
                                    .unwrap_or_else(|e| vec![Err(AppError::from(e))])
                            })
                            .collect()
                    }
                    Err(e) => vec![Err(AppError::from(e))],
                };

                stream::iter(results)
            })
            .try_flatten();
        Ok(Box::pin(stream))
    }

    async fn create_text_gen_params(
        &self,
        context: AgentContext,
        chat_id: Uuid,
    ) -> Result<Option<Self::TextGenParams>, AppError> {
        let config = context.agent_repo.get_agent_config(self.id).await?;

        Ok(match config {
            Some(config) => {
                let chat_messages = context.chat_repo.get_chat_messages(chat_id).await?;
                Some(Self::TextGenParams {
                    api_key: context
                        .cipher
                        .decrypt_base64_str(&config.api_key.unwrap_or_default())?,
                    messages: chat_messages
                        .into_iter()
                        .map(|a| GoogleTextGenParamsMessage {
                            role: match a.role.as_str() {
                                "model" => "model",
                                _ => "user",
                            }
                            .into(),
                            content: a.content,
                        })
                        .collect(),
                })
            }
            None => None,
        })
    }
}

impl AgentTextGenParamsApi for GoogleTextGenParams {
    fn push_message_str(&mut self, message: &str) {
        self.messages.push(GoogleTextGenParamsMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });
    }
}
