use crate::client::ClientConfig;
#[allow(unused_imports)]
use crate::error::{Error, Result};
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};
use futures::stream::Stream;
use reqwest::Client;

/// Public chat API, providing methods for chat completions.
pub struct ChatApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl ChatApi {
    /// Creates a new ChatApi instance given a reqwest client and a client configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        // Clone the config because ChatApi owns its own copy.
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Example method for chat completion that returns a complete response.
    pub async fn chat_completion(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        // Here you would implement your chat completion logic using self.client.
        // For now, we leave it unimplemented.
        unimplemented!()
    }

    /// Example method that returns a stream of chat completion chunks.
    pub fn chat_completion_stream(
        &self,
        _request: ChatCompletionRequest,
    ) -> impl Stream<Item = Result<ChatCompletionChunk>> {
        // Implement a streaming version. For now we return an empty stream.
        futures::stream::empty()
    }
}
