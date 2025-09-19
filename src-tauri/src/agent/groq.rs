use std::pin::Pin;

use async_trait::async_trait;
use futures::{stream, TryStreamExt};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use tokio_util::{codec::FramedRead, io::StreamReader};
use uuid::Uuid;

use crate::{
    agent::{AgentApi, AgentContext, AgentTextGenParamsApi, AgentTextGenResult},
    codec::sse::SseDecoder,
    common::error::AppError,
};

const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_API_KEY: &str = "Authorization";

pub struct GroqAgent {
    pub id: Uuid,
    pub model: String,
}

pub struct GroqTextGenParams {
    pub api_key: String,
    pub messages: Vec<GroqTextGenParamsMessage>,
}

pub struct GroqTextGenParamsMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct GroqTextGenRequestBody {
    pub messages: Vec<GroqTextGenRequestBodyMessage>,
    pub model: String,
    pub stream: bool,
    pub include_reasoning: bool,
}

#[derive(Serialize)]
pub struct GroqTextGenRequestBodyMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct GroqTextGenResponseBody {
    // pub id: String,
    // pub object: String,
    // pub created: u64,
    // pub model: String,
    // pub system_fingerint: String,
    pub choices: Vec<GroqTextGenResponseBodyChoice>,
}

#[derive(Deserialize)]
pub struct GroqTextGenResponseBodyChoice {
    // pub index: u64,
    pub delta: GroqTextGenResponseBodyChoiceDelta,
    // pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct GroqTextGenResponseBodyChoiceDelta {
    pub content: Option<String>,
}

#[async_trait]
impl AgentApi for GroqAgent {
    type TextGenParams = GroqTextGenParams;

    async fn generate_text(
        self,
        context: AgentContext,
        params: Self::TextGenParams,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentTextGenResult, AppError>> + Send>>, AppError>
    {
        let client = context.http_client_manager.get_client();
        let body = GroqTextGenRequestBody {
            messages: params
                .messages
                .iter()
                .map(|a| GroqTextGenRequestBodyMessage {
                    role: a.role.clone(),
                    content: a.content.clone(),
                })
                .collect(),
            model: self.model,
            stream: true,
            include_reasoning: false,
        };
        println!("REQUEST BODY: {:?}", serde_json::to_string(&body));
        let stream = client
            .request(
                reqwest::Method::POST,
                "https://api.groq.com/openai/v1/chat/completions",
            )
            .header(HEADER_CONTENT_TYPE, "application/json")
            .header(HEADER_API_KEY, format!("Bearer {}", params.api_key))
            .json(&body)
            .send()
            .await
            .map_err(AppError::from)?
            .bytes_stream();
        let reader = StreamReader::new(
            stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
        );
        let framed_stream = FramedRead::new(reader, SseDecoder::new())
            .map_ok(|line| {
                println!("GROQ LINE: {}", line);
                let data = match line.strip_prefix("data: ") {
                    Some(d) => d,
                    None => return stream::iter(vec![]),
                };

                if data == "[DONE]" {
                    return stream::iter(vec![]);
                }

                let response_body =
                    serde_json::from_str::<GroqTextGenResponseBody>(data).map_err(AppError::from);

                let results: Vec<Result<AgentTextGenResult, AppError>> = match response_body {
                    Ok(body) => body
                        .choices
                        .into_iter()
                        .filter_map(|a| {
                            a.delta
                                .content
                                .map(|content| Ok(AgentTextGenResult { text: content }))
                        })
                        .collect(),
                    Err(e) => vec![Err(e)],
                };
                stream::iter(results)
            })
            .try_flatten();
        Ok(Box::pin(framed_stream))
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
                        .map(|a| GroqTextGenParamsMessage {
                            role: match a.role.as_str() {
                                "user" => "user".to_string(),
                                "model" => "assistant".to_string(),
                                _ => panic!("unknown role"),
                            },
                            content: a.content,
                        })
                        .collect(),
                })
            }
            None => None,
        })
    }
}

impl AgentTextGenParamsApi for GroqTextGenParams {
    fn push_message_str(&mut self, message: &str) {
        self.messages.push(GroqTextGenParamsMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });
    }
}
