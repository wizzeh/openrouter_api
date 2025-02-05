// api/completion.rs
use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::completion::{CompletionRequest, CompletionResponse};
use reqwest::Client;

/// API endpoint for text completions.
pub struct CompletionApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl CompletionApi {
    /// Creates a new CompletionApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Calls the completions endpoint. The request payload includes at minimum the `model` and `prompt` fields,
    /// along with any additional generation parameters (temperature, top_p, and so on).
    pub async fn text_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Build the URL to join the base_url for completions.
        let url = self
            .config
            .base_url
            .join("completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for completions: {}", e),
                metadata: None,
            })?;

        let response = self
            .client
            .post(url)
            .headers(self.config.build_headers()?)
            .json(&request)
            .send()
            .await?;

        // Check the response status; if it's an error, create our custom Error.
        if !response.status().is_success() {
            return Err(Error::ApiError {
                code: response.status().as_u16(),
                message: response.text().await?,
                metadata: None,
            });
        }

        let body = response.text().await?;
        if body.trim().is_empty() {
            return Err(Error::ApiError {
                code: response.status().as_u16(),
                message: "Empty response body".into(),
                metadata: None,
            });
        }

        serde_json::from_str::<CompletionResponse>(&body).map_err(|e| Error::ApiError {
            code: response.status().as_u16(),
            message: format!("Failed to decode JSON: {}. Body was: {}", e, body),
            metadata: None,
        })
    }
}
