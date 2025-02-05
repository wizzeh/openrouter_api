/* src/api/chat.rs */

use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};
use async_stream::try_stream;
use futures::stream::Stream;
use futures::StreamExt;
use futures::TryStreamExt; // Required for map_err on bytes_stream.
use reqwest::Client;
use std::pin::Pin;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;

pub struct ChatApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl ChatApi {
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    pub async fn chat_completion(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        // Implementation omitted for brevity.
        unimplemented!()
    }

    pub fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>> {
        let client = self.client.clone();
        let config = self.config.clone();

        let stream = try_stream! {
            // Build the complete URL for the chat completions endpoint.
            let url = config.base_url.join("chat/completions").map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL: {}", e),
                metadata: None,
            })?;

            // Serialize the request into a JSON value.
            let mut req_body = serde_json::to_value(&request).map_err(|e| Error::ApiError {
                code: 500,
                message: format!("Request serialization error: {}", e),
                metadata: None,
            })?;
            // Ensure streaming is enabled.
            req_body["stream"] = serde_json::Value::Bool(true);

            // Issue the POST request with the appropriate headers and JSON body.
            // Use error_for_status() to perform status checking without consuming the response twice.
            let response = client
                .post(url)
                .headers(config.build_headers()?)
                .json(&req_body)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| {
                    // Map the reqwest error into our custom Error.
                    Error::ApiError {
                        code: e.status().map(|s| s.as_u16()).unwrap_or(500),
                        message: e.to_string(),
                        metadata: None,
                    }
                })?;

            // Convert the response bytes stream into an asynchronous line stream.
            // At this point, the response is known to be successful.
            let byte_stream = response.bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let stream_reader = StreamReader::new(byte_stream);
            let mut lines = FramedRead::new(stream_reader, LinesCodec::new());

            // Process each SSE line.
            while let Some(line_result) = lines.next().await {
                // Map any LinesCodec error into our API error.
                let line = line_result.map_err(|e| Error::ApiError {
                    code: 500,
                    message: format!("LinesCodec error: {}", e),
                    metadata: None,
                })?;
                if line.trim().is_empty() {
                    continue;
                }
                if line.starts_with("data:") {
                    let data_part = line.trim_start_matches("data:").trim();
                    if data_part == "[DONE]" {
                        break;
                    }
                    match serde_json::from_str::<ChatCompletionChunk>(data_part) {
                        Ok(chunk) => yield chunk,
                        Err(_err) => continue,
                    }
                } else if line.starts_with(":") {
                    // SSE comment; ignore the line.
                    continue;
                }
            }
        };

        Box::pin(stream)
    }
}
