// openrouter_api/src/client.rs

#![allow(unused)]
// Fix for unused imports in src/client.rs
use crate::error::{Error, Result};
use crate::types;
use crate::types::routing::{PredefinedModelCoverageProfile, RouterConfig};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;

// [rest of client.rs remains the same]

/// Client configuration containing API key, base URL, and additional settings.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub user_id: Option<String>,
    pub timeout: Duration,
    pub retry_config: RetryConfig,
}

/// Configuration for automatic retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub retry_on_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 500,
            max_backoff_ms: 10000,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
        }
    }
}

impl ClientConfig {
    /// Build HTTP headers required for making API calls.
    /// Returns an error if any header value cannot be constructed.
    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            let auth_header = HeaderValue::from_str(&format!("Bearer {}", key))
                .map_err(|e| Error::ConfigError(format!("Invalid API key header format: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        // Content-Type header is always valid.
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref referer) = self.http_referer {
            let ref_value = HeaderValue::from_str(referer)
                .map_err(|e| Error::ConfigError(format!("Invalid Referer header: {}", e)))?;
            headers.insert("Referer", ref_value);
        }
        if let Some(ref title) = self.site_title {
            let title_value = HeaderValue::from_str(title)
                .map_err(|e| Error::ConfigError(format!("Invalid Title header: {}", e)))?;
            headers.insert("X-Title", title_value);
        }
        if let Some(ref user_id) = self.user_id {
            let user_id_value = HeaderValue::from_str(user_id)
                .map_err(|e| Error::ConfigError(format!("Invalid User-ID header: {}", e)))?;
            headers.insert("X-User-ID", user_id_value);
        }
        Ok(headers)
    }
}

// Type‑state markers.
pub struct Unconfigured;
pub struct NoAuth;
pub struct Ready;

/// Main OpenRouter client using a type‑state builder pattern.
pub struct OpenRouterClient<State = Unconfigured> {
    pub config: ClientConfig,
    pub http_client: Option<reqwest::Client>,
    pub _state: PhantomData<State>,
    pub router_config: Option<RouterConfig>,
}

impl OpenRouterClient<Unconfigured> {
    /// Creates a new unconfigured client.
    pub fn new() -> Self {
        Self {
            config: ClientConfig {
                api_key: None,
                // Default base URL; can be overridden with with_base_url().
                base_url: "https://openrouter.ai/api/v1/".parse().unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None,
                timeout: Duration::from_secs(30),
                retry_config: RetryConfig::default(),
            },
            http_client: None,
            _state: PhantomData,
            router_config: None,
        }
    }

    /// Sets the base URL and transitions to the NoAuth state.
    /// The base URL must include a trailing slash.
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
            http_client: None,
            _state: PhantomData,
            router_config: self.router_config,
        }
    }
}

impl OpenRouterClient<NoAuth> {
    /// Supplies the API key and transitions to the Ready state.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(api_key.into());
        self.transition_to_ready()
    }

    /// Optionally sets the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Optionally sets the HTTP referer header.
    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.config.http_referer = Some(referer.into());
        self
    }

    /// Optionally sets the site title header.
    pub fn with_site_title(mut self, title: impl Into<String>) -> Self {
        self.config.site_title = Some(title.into());
        self
    }

    /// Optionally sets the user ID header for tracking specific users.
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.config.user_id = Some(user_id.into());
        self
    }

    /// Optionally configures retry behavior.
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.config.retry_config = retry_config;
        self
    }

    /// Configures Model Coverage Profile for model selection and routing.
    pub fn with_model_coverage_profile(mut self, profile: PredefinedModelCoverageProfile) -> Self {
        self.router_config = Some(RouterConfig {
            profile,
            provider_preferences: None,
        });
        self
    }

    fn transition_to_ready(self) -> Result<OpenRouterClient<Ready>> {
        let headers = self.config.build_headers()?;
        
        // Build a client with retry capabilities
        let client_builder = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .default_headers(headers);
        
        let http_client = client_builder
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
            router_config: self.router_config,
        })
    }
}

impl OpenRouterClient<Ready> {
    /// Provides access to the chat endpoint.
    pub fn chat(&self) -> Result<crate::api::chat::ChatApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::chat::ChatApi::new(client, &self.config))
    }

    /// Provides access to the completions endpoint.
    pub fn completions(&self) -> Result<crate::api::completion::CompletionApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::completion::CompletionApi::new(client, &self.config))
    }

    /// Provides access to the models endpoint.
    pub fn models(&self) -> Result<crate::api::models::ModelsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::models::ModelsApi::new(client, &self.config))
    }

    /// Provides access to the structured output endpoint.
    pub fn structured(&self) -> Result<crate::api::structured::StructuredApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::structured::StructuredApi::new(client, &self.config))
    }

    /// Provides access to the web search endpoint.
    pub fn web_search(&self) -> Result<crate::api::web_search::WebSearchApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::web_search::WebSearchApi::new(
            client,
            &self.config,
        ))
    }

    /// Returns a new request builder for chat completions that supports MCP.
    pub fn chat_request_builder(
        &self,
        messages: Vec<crate::types::chat::Message>,
    ) -> crate::api::request::RequestBuilder<serde_json::Value> {
        // Apply the model coverage profile if available
        let primary_model = if let Some(router_config) = &self.router_config {
            match &router_config.profile {
                PredefinedModelCoverageProfile::Custom(profile) => profile.primary.clone(),
                PredefinedModelCoverageProfile::LowestLatency => "openai/gpt-3.5-turbo".to_string(),
                PredefinedModelCoverageProfile::LowestCost => "openai/gpt-3.5-turbo".to_string(),
                PredefinedModelCoverageProfile::HighestQuality => "anthropic/claude-3-opus-20240229".to_string(),
            }
        } else {
            "openai/gpt-4o".to_string()
        };
        
        // Set up basic params
        let mut extra_params = serde_json::json!({});
        
        // Add provider preferences if set
        if let Some(router_config) = &self.router_config {
            if let Some(provider_prefs) = &router_config.provider_preferences {
                extra_params["provider"] = serde_json::to_value(provider_prefs).unwrap_or_default();
            }
            
            // Add fallback models if present in custom profile
            if let PredefinedModelCoverageProfile::Custom(profile) = &router_config.profile {
                if let Some(fallbacks) = &profile.fallbacks {
                    extra_params["models"] = serde_json::to_value(fallbacks).unwrap_or_default();
                }
            }
        }
        
        crate::api::request::RequestBuilder::new(primary_model, messages, extra_params)
    }

    /// Helper method to handle standard HTTP responses.
    pub(crate) async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
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

    /// Validates tool calls in a chat completion response.
    pub fn validate_tool_calls(
        &self,
        response: &crate::types::chat::ChatCompletionResponse,
    ) -> Result<()> {
        for choice in &response.choices {
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tc in tool_calls {
                    if tc.kind != "function" {
                        return Err(Error::SchemaValidationError(format!(
                            "Invalid tool call kind: {}. Expected 'function'",
                            tc.kind
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

