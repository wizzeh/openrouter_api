use crate::error::{Error, Result};
use crate::types;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    // Ensure the base URL ends with a trailing slash.
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        // Use trailing slash so that join works as expected.
        Self {
            api_key: None,
            base_url: Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            http_referer: None,
            site_title: None,
            timeout: Duration::from_secs(30),
        }
    }
}

impl ClientConfig {
    /// Build HTTP headers required for making API calls.
    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key.as_ref().unwrap()))
                .expect("Invalid API key header"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref referer) = self.http_referer {
            headers.insert("HTTP-Referer", HeaderValue::from_str(referer).unwrap());
        }
        if let Some(ref title) = self.site_title {
            headers.insert("X-Title", HeaderValue::from_str(title).unwrap());
        }
        headers
    }
}

// Type‑state markers.
pub struct Unconfigured;
pub struct NoAuth;
pub struct Ready;

/// The main OpenRouter client using a type‑state builder pattern.
pub struct OpenRouterClient<State = Unconfigured> {
    pub config: ClientConfig,
    pub http_client: Option<reqwest::Client>,
    pub _state: PhantomData<State>,
}

impl OpenRouterClient<Unconfigured> {
    /// Create a new unconfigured client.
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
            http_client: None,
            _state: PhantomData,
        }
    }

    /// Set the base URL and transition to the NoAuth state.
    /// The base URL should include a trailing slash (e.g., "https://openrouter.ai/api/v1/").
    pub fn with_base_url(
        mut self,
        base_url: impl Into<String>,
    ) -> Result<OpenRouterClient<NoAuth>> {
        self.config.base_url = Url::parse(&base_url.into()).map_err(|e| Error::ApiError {
            code: 400,
            message: format!("Invalid base URL: {}", e),
            metadata: None,
        })?;
        Ok(self.transition_to_no_auth())
    }

    fn transition_to_no_auth(self) -> OpenRouterClient<NoAuth> {
        OpenRouterClient {
            config: self.config,
            http_client: self.http_client,
            _state: PhantomData,
        }
    }
}

impl OpenRouterClient<NoAuth> {
    /// Supply the API key and transition to the Ready state.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> OpenRouterClient<Ready> {
        self.config.api_key = Some(api_key.into());
        self.transition_to_ready()
    }

    /// Optionally set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Optionally set the HTTP referer header.
    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.config.http_referer = Some(referer.into());
        self
    }

    /// Optionally set the site title header.
    pub fn with_site_title(mut self, title: impl Into<String>) -> Self {
        self.config.site_title = Some(title.into());
        self
    }

    fn transition_to_ready(self) -> OpenRouterClient<Ready> {
        let http_client = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
        }
    }
}

impl OpenRouterClient<Ready> {
    /// Call the chat completions API.
    pub async fn chat_completion(
        &self,
        request: types::chat::ChatCompletionRequest,
    ) -> Result<types::chat::ChatCompletionResponse> {
        // Notice: join "chat/completions" as a relative path; the base URL includes "v1/".
        let url = self
            .config
            .base_url
            .join("chat/completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("URL join error: {}", e),
                metadata: None,
            })?;

        let response = self
            .http_client
            .as_ref()
            .unwrap()
            .post(url)
            .headers(self.config.build_headers())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ApiError {
                code: response.status().as_u16(),
                message: response.text().await?,
                metadata: None,
            });
        }
        self.handle_response(response).await
    }

    /// Handle the response by reading text and parsing JSON.
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: body.clone(),
                metadata: None,
            });
        }
        if body.trim().is_empty() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: "Empty response body".into(),
                metadata: None,
            });
        }
        serde_json::from_str::<T>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: format!("Failed to decode JSON: {}. Body was: {}", e, body),
            metadata: None,
        })
    }
}
