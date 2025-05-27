//// File: openrouter_api/src/tests/integration_tests.rs
/*
   src/tests/integration_tests.rs

   Integration tests for the OpenRouter client.
*/

#[cfg(test)]
mod integration_tests {
    use crate::client::{OpenRouterClient, RetryConfig, Unconfigured};
    #[allow(unused_imports)]
    use crate::models::chat::{ChatMessage, ChatRole};
    #[allow(unused_imports)]
    use crate::models::provider_preferences::{
        DataCollection, ProviderPreferences, ProviderSort, Quantization,
    };
    #[allow(unused_imports)]
    use crate::models::structured::{JsonSchemaConfig, JsonSchemaDefinition};
    #[allow(unused_imports)]
    use crate::models::tool::{FunctionCall, FunctionDescription, Tool, ToolCall};
    use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, Message};
    use serde_json::{json, Value};
    use std::env;
    use url::Url;

    // Helper function to deserialize a ChatCompletionResponse from JSON.
    fn deserialize_chat_response(json_str: &str) -> ChatCompletionResponse {
        serde_json::from_str::<ChatCompletionResponse>(json_str).expect("Valid JSON")
    }

    #[tokio::test]
    async fn test_basic_chat_completion() -> Result<(), Box<dyn std::error::Error>> {
        // Read the API key from the environment.
        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|e| format!("OPENROUTER_API_KEY must be set in the environment: {}", e))?;

        // Build the client: Unconfigured -> NoAuth -> Ready.
        let _client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_http_referer("https://github.com/your_org/your_repo")
            .with_site_title("OpenRouter Rust SDK Tests")
            .with_api_key(api_key);

        // Create a basic chat completion request.
        let _request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "What is a phantom type in Rust?".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            stream: None,
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
        };

        // For this integration test we are simulating a response.
        let simulated_response_json = r#"
        {
            "id": "gen-123",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "A phantom type is a type parameter that is not used in any fields.",
                    "tool_calls": null
                },
                "finish_reason": "stop",
                "native_finish_reason": "stop"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o",
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);
        assert!(!response.choices.is_empty());
        assert_eq!(response.choices[0].message.role, "assistant");

        Ok(())
    }

    #[tokio::test]
    async fn test_valid_tool_call_response() -> Result<(), Box<dyn std::error::Error>> {
        // Simulate a valid ChatCompletionResponse with a proper tool call.
        let simulated_response_json = r#"
        {
            "id": "gen-valid-tool",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Calling tool for weather.",
                    "tool_calls": [{
                        "id": "call-001",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"Boston\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls",
                "native_finish_reason": "tool_calls"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o"
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);

        // Create a dummy client in Ready state to call our validation helper.
        let client = OpenRouterClient::<crate::client::Ready> {
            config: crate::client::ClientConfig {
                api_key: Some("dummy".into()),
                base_url: Url::parse("https://dummy/").unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None, // Add this field
                timeout: std::time::Duration::from_secs(30),
                retry_config: RetryConfig::default(), // Add this field
            },
            http_client: None,
            _state: std::marker::PhantomData,
            router_config: None, // Add this field
        };

        // Validate the tool calls – should return Ok.
        client.validate_tool_calls(&response)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_text_completion_response_deserialization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simulated response JSON from the text completion endpoint.
        let simulated_response_json = r#"
        {
            "id": "comp-123",
            "choices": [
                {
                    "text": "Once upon a time, in a land far, far away...",
                    "index": 0,
                    "finish_reason": "stop"
                }
            ]
        }
        "#;

        // Deserialize the response.
        let response = serde_json::from_str::<crate::types::completion::CompletionResponse>(
            simulated_response_json,
        )?;

        // Verify that the deserialization worked correctly.
        assert!(response.choices.len() > 0);
        assert_eq!(response.choices[0].finish_reason.as_deref(), Some("stop"));
        assert!(response.choices[0].text.contains("Once upon a time"));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_tool_call_response() -> Result<(), Box<dyn std::error::Error>> {
        // Simulate an invalid ChatCompletionResponse where the tool call kind is not "function".
        let simulated_response_json = r#"
        {
            "id": "gen-invalid-tool",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Invalid tool call.",
                    "tool_calls": [{
                        "id": "call-002",
                        "type": "invalid",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"Boston\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls",
                "native_finish_reason": "tool_calls"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o"
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);

        // Create a dummy client to perform validation.
        let client = OpenRouterClient::<crate::client::Ready> {
            config: crate::client::ClientConfig {
                api_key: Some("dummy".into()),
                base_url: Url::parse("https://dummy/").unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None, // Add this field
                timeout: std::time::Duration::from_secs(30),
                retry_config: RetryConfig::default(), // Add this field
            },
            http_client: None,
            _state: std::marker::PhantomData,
            router_config: None, // Add this field
        };

        // Validate the tool calls – should return a SchemaValidationError.
        let validation_result = client.validate_tool_calls(&response);
        assert!(validation_result.is_err());
        if let Err(err) = validation_result {
            match err {
                crate::error::Error::SchemaValidationError(msg) => {
                    assert!(msg.contains("Invalid tool call kind"));
                }
                _ => panic!("Expected a SchemaValidationError"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_provider_preferences_serialization() -> Result<(), Box<dyn std::error::Error>> {
        // Build a provider preferences configuration.
        let preferences = crate::models::provider_preferences::ProviderPreferences {
            order: Some(vec!["OpenAI".to_string(), "Anthropic".to_string()]),
            allow_fallbacks: Some(false),
            require_parameters: Some(true),
            data_collection: Some(crate::models::provider_preferences::DataCollection::Deny),
            ignore: Some(vec!["Azure".to_string()]),
            quantizations: Some(vec![
                crate::models::provider_preferences::Quantization::Fp8,
                crate::models::provider_preferences::Quantization::Int8,
            ]),
            sort: Some(crate::models::provider_preferences::ProviderSort::Throughput),
        };

        // Start with an empty extra parameters object.
        let extra_params = json!({});

        // Use the request builder to attach the provider preferences.
        let builder =
            crate::api::request::RequestBuilder::new("openai/gpt-4o", vec![], extra_params)
                .with_provider_preferences(preferences)
                .expect("Provider preferences should be valid");

        // Serialize the complete payload.
        let payload = builder.build();
        let payload_json = serde_json::to_string_pretty(&payload)?;
        println!("Payload with provider preferences:\n{}", payload_json);

        // Check that the serialized JSON contains the "provider" key with the expected configuration.
        let payload_value: Value = serde_json::from_str(&payload_json)?;
        let provider_config = payload_value.get("provider").expect("provider key missing");
        assert_eq!(provider_config.get("allowFallbacks").unwrap(), false);
        assert_eq!(provider_config.get("sort").unwrap(), "throughput");

        Ok(())
    }

    #[tokio::test]
    async fn test_web_search_response_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        // Simulated web search response JSON.
        let simulated_response_json = r#"
        {
            "query": "rust programming",
            "results": [
                {
                    "title": "The Rust Programming Language",
                    "url": "https://www.rust-lang.org",
                    "snippet": "Learn Rust programming."
                },
                {
                    "title": "Rust by Example",
                    "url": "https://doc.rust-lang.org/rust-by-example/",
                    "snippet": "A collection of runnable examples."
                }
            ],
            "total_results": 2
        }
        "#;
        let response: crate::types::web_search::WebSearchResponse =
            serde_json::from_str(simulated_response_json)?;
        assert_eq!(response.query, "rust programming");
        assert_eq!(response.total_results, 2);
        assert_eq!(response.results.len(), 2);
        Ok(())
    }
}
