/*
   src/tests/integration_tests.rs

   Integration tests for the OpenRouter client.
*/

#[cfg(test)]
mod integration_tests {
    use crate::client::{OpenRouterClient, Unconfigured};
    use crate::models::chat::{ChatMessage, ChatRole};
    use crate::models::structured::{JsonSchemaConfig, JsonSchemaDefinition};
    use crate::models::tool::{FunctionCall, FunctionDescription, Tool, ToolCall};
    use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, Message};
    use serde_json::json;
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
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_http_referer("https://github.com/your_org/your_repo")
            .with_site_title("OpenRouter Rust SDK Tests")
            .with_api_key(api_key);

        // Create a basic chat completion request.
        let request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "What is a phantom type in Rust?".to_string(),
                name: None,
                tool_calls: None,
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
                timeout: std::time::Duration::from_secs(30),
            },
            http_client: None,
            _state: std::marker::PhantomData,
        };

        // Validate the tool calls – should return Ok.
        client.validate_tool_calls(&response)?;

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
                timeout: std::time::Duration::from_secs(30),
            },
            http_client: None,
            _state: std::marker::PhantomData,
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
    async fn test_structured_output_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Parse the base URL.
        let base_url: Url = "https://api.example.com/".parse()?;

        // Build the client: Unconfigured -> NoAuth -> Ready.
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url(base_url)?
            .with_api_key("dummy_key");

        // Prepare a simple chat message using ChatMessage (models) and convert to Message.
        let chat_messages = vec![crate::models::chat::ChatMessage {
            role: crate::models::chat::ChatRole::User,
            content: "What is the weather like?".into(),
        }];
        let messages: Vec<Message> = chat_messages.into_iter().map(Into::into).collect();

        // Build a JSON Schema definition for the expected structured output.
        let schema_def = JsonSchemaDefinition {
            schema_type: "object".into(),
            properties: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "location".into(),
                    json!({"type": "string", "description": "City or location name"}),
                );
                map.insert(
                    "temperature".into(),
                    json!({"type": "number", "description": "Temperature in Celsius"}),
                );
                map.insert(
                    "conditions".into(),
                    json!({"type": "string", "description": "Weather conditions description"}),
                );
                map
            },
            required: Some(vec![
                "location".into(),
                "temperature".into(),
                "conditions".into(),
            ]),
            additional_properties: Some(false),
        };

        let json_schema_config = JsonSchemaConfig {
            name: "weather".into(),
            strict: true,
            schema: schema_def,
        };

        // Build a completion request with structured output enabled.
        let request_payload = client
            .completion_request(messages)
            .with_structured_output(json_schema_config, true, false)
            .build();

        // Serialize the payload to JSON.
        let payload_json = serde_json::to_string_pretty(&request_payload)?;
        println!("Structured Request payload:\n{}", payload_json);

        // In a full integration test, you'd send this payload, receive and validate the response against the JSON Schema.

        Ok(())
    }
}
