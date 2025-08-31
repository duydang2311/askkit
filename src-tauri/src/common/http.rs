use reqwest::{Client, RequestBuilder};
use std::sync::OnceLock;

#[derive(Clone)]
pub struct HttpClientManager {
    client: OnceLock<Client>,
}

impl HttpClientManager {
    pub fn new() -> Self {
        Self {
            client: OnceLock::new(),
        }
    }

    pub fn get_client(&self) -> &Client {
        self.client.get_or_init(|| Client::new())
    }
}

pub trait BuildAgentPromptRequest {
    fn build_prompt_request(&self, client: &Client) -> RequestBuilder;
}
