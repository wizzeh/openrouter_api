#[allow(unused_imports)]
use crate::client::{OpenRouterClient, Unconfigured};
#[allow(unused_imports)]
use crate::models::chat::{ChatMessage, ChatRole};
#[allow(unused_imports)]
use crate::models::structured::{JsonSchemaConfig, JsonSchemaDefinition};
#[allow(unused_imports)]
use crate::types::{ChatCompletionRequest, CompletionRequest, Message};
#[allow(unused_imports)]
use serde_json::json;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use url::Url;

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
        }],
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    // Send the chat completion request.
    let response = client.chat_completion(request).await?;

    println!("Model used: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }
    println!("Usage: {:?}", response.usage);

    // Perform some basic assertions.
    assert!(!response.choices.is_empty());
    assert_eq!(response.choices[0].message.role, "assistant");

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

    // Prepare a simple chat message.
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: "What is the weather like?".into(),
    }];

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

    // Serialize the payload to JSON without unwrap().
    let payload_json = serde_json::to_string_pretty(&request_payload)?;
    println!("Structured Request payload:\n{}", payload_json);

    // In a full integration test, you'd send this payload, receive and validate the response against the JSON Schema.

    Ok(())
}
