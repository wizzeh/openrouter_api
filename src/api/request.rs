/*
   src/api/request.rs

   This module implements the unified request builder for non‑interactive endpoints,
   supporting structured outputs and tool calling.
*/

use crate::models::structured::JsonSchemaConfig;
use crate::models::tool::Tool;
use crate::types::chat::Message;
use serde::Serialize;
#[allow(unused_imports)]
use serde_json::Value;

/// Representation for enabling structured outputs in the request payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormatConfig {
    /// Fixed string indicating the type of response format.
    #[serde(rename = "type")]
    pub format_type: String,
    /// The JSON Schema configuration.
    pub json_schema: JsonSchemaConfig,
}

/// Payload sent to the API. It is generic over extra parameters.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPayload<T: Serialize> {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormatConfig>,
    /// Optional tool calling instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(flatten)]
    pub extra_params: T,
}

/// A unified request builder for non‑interactive endpoints supporting structured outputs
/// and tool calling.
pub struct RequestBuilder<T: Serialize> {
    model: String,
    messages: Vec<Message>,
    extra_params: T,
    structured_output: Option<ResponseFormatConfig>,
    /// Optional tools for tool calling.
    tools: Option<Vec<Tool>>,
    /// Indicates whether to perform JSON Schema validation on the response.
    pub validate_structured: bool,
    /// If true, fallback to unstructured output on validation failure.
    pub fallback_on_failure: bool,
}

impl<T: Serialize> RequestBuilder<T> {
    /// Creates a new request builder.
    pub fn new(model: impl Into<String>, messages: Vec<Message>, extra_params: T) -> Self {
        Self {
            model: model.into(),
            messages,
            extra_params,
            structured_output: None,
            tools: None,
            validate_structured: true,
            fallback_on_failure: false,
        }
    }

    /// Enables structured output support.
    /// `validate` sets whether to perform JSON Schema validation,
    /// `fallback` sets whether to fallback to an unstructured response on failure.
    pub fn with_structured_output(
        mut self,
        config: JsonSchemaConfig,
        validate: bool,
        fallback: bool,
    ) -> Self {
        self.structured_output = Some(ResponseFormatConfig {
            format_type: "json_schema".to_string(),
            json_schema: config,
        });
        self.validate_structured = validate;
        self.fallback_on_failure = fallback;
        self
    }

    /// Enables tool calling by providing a list of tools.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Consumes the builder and returns the complete request payload.
    pub fn build(self) -> RequestPayload<T> {
        RequestPayload {
            model: self.model,
            messages: self.messages,
            response_format: self.structured_output,
            tools: self.tools,
            extra_params: self.extra_params,
        }
    }
}
