use reqwest::{Client, RequestBuilder};

use crate::common::http::BuildAgentPromptRequest;

pub enum Agent {
    Gemini(GeminiAgent),
}

pub struct GeminiAgent {
    pub id: String,
    pub api_key: String,
    pub model: String,
}

impl BuildAgentPromptRequest for GeminiAgent {
    fn build_prompt_request(&self, client: &Client) -> RequestBuilder {
        client.request(reqwest::Method::POST, format!("https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse", &self.model))
            .header("Content-Type", "application/json")
            .header("X-goog-api-key", &self.api_key)
    }
}

impl BuildAgentPromptRequest for Agent {
    fn build_prompt_request(&self, client: &Client) -> RequestBuilder {
        match self {
            Agent::Gemini(agent) => agent.build_prompt_request(client),
        }
    }
}
