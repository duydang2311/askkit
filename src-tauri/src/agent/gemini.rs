use aes_gcm::aes::cipher;
use async_trait::async_trait;
use futures_util::{self, stream, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::{AgentApi, AgentContext, AgentTextGenParamsApi, AgentTextGenResult};
use crate::common::error::AppError;

const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_X_GOOG_API_KEY: &str = "X-goog-api-key";

#[derive(Clone)]
pub struct GeminiAgent {
    pub id: Uuid,
    pub model: String,
}

pub struct GeminiTextGenParams {
    pub api_key: String,
    pub messages: Vec<GeminiTextGenParamsMessage>,
}

pub struct GeminiTextGenParamsMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct GeminiTextGenRequestBody {
    pub contents: Vec<GeminiTextGenRequestBodyContent>,
}

#[derive(Serialize)]
pub struct GeminiTextGenRequestBodyContent {
    pub role: String,
    pub parts: Vec<GeminiTextGenRequestBodyContentPart>,
}

#[derive(Serialize)]
pub struct GeminiTextGenRequestBodyContentPart {
    pub text: String,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBody {
    candidates: Vec<GeminiTextGenResponseBodyCandidate>,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBodyCandidate {
    pub content: GeminiTextGenResponseBodyCandidateContent,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBodyCandidateContent {
    pub parts: Vec<GeminiTextGenResponseBodyCandidateContentPart>,
}

#[derive(Deserialize)]
pub struct GeminiTextGenResponseBodyCandidateContentPart {
    pub text: String,
}

#[async_trait]
impl AgentApi for GeminiAgent {
    type TextGenParams = GeminiTextGenParams;

    async fn generate_text(
        self,
        context: AgentContext,
        params: Self::TextGenParams,
    ) -> Result<impl Stream<Item = Result<AgentTextGenResult, AppError>>, AppError> {
        let client = context.http_client_manager.get_client();
        let body = GeminiTextGenRequestBody {
            contents: params
                .messages
                .into_iter()
                .map(|a| GeminiTextGenRequestBodyContent {
                    role: a.role,
                    parts: vec![GeminiTextGenRequestBodyContentPart { text: a.content }],
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
        Ok(stream)
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
                        .map(|a| GeminiTextGenParamsMessage {
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

impl AgentTextGenParamsApi for GeminiTextGenParams {
    type Message = GeminiTextGenParamsMessage;

    fn push_message(&mut self, message: Self::Message) {
        self.messages.push(message);
    }

    fn push_message_str(&mut self, message: &str) {
        self.messages.push(Self::Message {
            role: "user".to_string(),
            content: message.to_string(),
        });
    }
}
