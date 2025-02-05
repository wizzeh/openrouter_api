use serde::{Deserialize, Serialize};

/// A chat message for a chat completion request.
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat completion request matching the OpenRouter API schema.
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    /// The model ID to use.
    pub model: String,
    /// The list of messages.
    pub messages: Vec<Message>,
    /// Whether the response should be streamed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// (Optional) Stub for response_format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// (Optional) Stub for tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// (Optional) Stub for provider preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// (Optional) Fallback models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// (Optional) Message transforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
}

/// A choice returned by the chat API.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
}

/// Usage data returned from the API.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Chat completion response.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    pub usage: Option<Usage>,
}

/// A streaming chunk for chat completions (stub).
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<Choice>,
}
