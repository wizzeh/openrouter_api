// src/api/chat.rs
use crate::error::Result;
use crate::types::*;

impl OpenRouterClient<Ready> {
    /// Creates a chat completion
    ///
    /// This endpoint is compatible with OpenAI's chat completion API.
    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = self.config.base_url.join("/chat/completions")?;

        let response = self
            .http_client
            .as_ref()
            .unwrap()
            .post(url)
            .json(&request)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_ref().unwrap()),
            )
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Creates a streaming chat completion
    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        // Implementation
    }
}
