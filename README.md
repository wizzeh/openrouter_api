# OpenRouter API Client Library

OpenRouter API Client Library is a Rust client for interfacing with the OpenRouter API. The library is designed to be modular, type‑safe, and intuitive. It uses a type‑state builder pattern for configuring and validating the client at compile time, ensuring that all required configuration (such as setting the base URL and API key) happens before attempting a request.

> **Note:** This project is still in development. Many features are planned but not yet fully implemented.

## Features

- **Modular Organization:** Organized into clear modules for models, API endpoints, common types, and utilities.
- **Type‑State Builder:** Guarantees compile‑time validation of client configuration (e.g. base URL, API key, custom headers) for a robust development experience.
- **HTTP Integration:** Uses [reqwest](https://crates.io/crates/reqwest) with rustls‑tls for secure asynchronous HTTP requests.
- **Robust Error Handling:** Centralized error management using the `thiserror` crate ensures consistent error types across the library.
- **Streaming Support:** Now supports streaming chat completions via Server-Sent Events (SSE). The library gracefully skips over comment lines and non‑JSON payloads, letting you update UIs in real‑time.
- **Structured Outputs:** Optionally request structured responses with JSON Schema validation so that responses strictly follow your defined schema.
- **Tool Calling Capability:** Define function‑type tools that the model can invoke. Supports concurrent tool calls in a single response with proper validation against expected formats.
- **Provider Preferences & Routing:** Configure model fallbacks, routing preferences, and provider filtering via a strongly‑typed interface.
- **Web Search Endpoint:** Easily perform web search queries with type‑safe request and response models.
- **Future Roadmap:**
  - Full Streaming support for real‑time chat completions (currently available for chat yet to be fully integrated in all endpoints).
  - Text completion endpoint.
  - Endpoints for credits, generation metadata, and available models.
  - Extended tests, enhanced documentation, and CI integration.

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
openrouter_api = { git = "https://github.com/yourusername/openrouter_api.git", branch = "main" }
```

Ensure that you have Rust installed (tested with Rust v1.83.0) and that you're using Cargo for building and testing.

### Example Usage

#### Minimal Chat Example

```rust
use openrouter_api::{OpenRouterClient, Ready, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure your API key is set in the environment.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Build the client (Unconfigured -> NoAuth -> Ready).
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a minimal chat completion request.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
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

    // Invoke the chat completion endpoint.
    let response = client.chat_completion(request).await?;

    // Output the model's response.
    if let Some(choice) = response.choices.first() {
        println!("Chat Response: {}", choice.message.content);
    }
    Ok(())
}
```

#### Minimal Web Search Example

```rust
use openrouter_api::{OpenRouterClient, Ready, Result};
use openrouter_api::types::web_search::{WebSearchRequest, WebSearchResponse};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure your API key is set in the environment.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Build the client (Unconfigured -> NoAuth -> Ready).
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a minimal web search request.
    let request = WebSearchRequest {
        query: "rust programming".into(),
        num_results: Some(5),
    };

    // Invoke the web search endpoint.
    let response: WebSearchResponse = client.web_search()?.search(request).await?;

    // Print out the search results.
    println!("Search query: {}", response.query);
    for result in response.results {
        println!("Title: {}\nURL: {}\n", result.title, result.url);
    }

    Ok(())
}
```

#### Streaming Chat Example

The library now supports streaming chat completions via SSE:

```rust
use openrouter_api::{OpenRouterClient, Ready, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message, ChatCompletionChunk};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure your API key is set in the environment.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Build the client.
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a chat completion request with streaming enabled.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Tell me a story.".to_string(),
            name: None,
            tool_calls: None,
        }],
        stream: Some(true),
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    // Invoke the streaming chat completion endpoint.
    let mut stream = client.chat()?.chat_completion_stream(request);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                // Incrementally output the content.
                print!("{}", c.message.content);
            }
            Err(e) => eprintln!("Error during streaming: {}", e),
        }
    }
    println!();
    Ok(())
}
```

#### Tool Calling & Structured Outputs Example

This example demonstrates how to include tool calling information and request structured output validation using a JSON Schema.

```rust
use openrouter_api::{
    OpenRouterClient, Ready,
    types::chat::{ChatCompletionRequest, Message},
    models::structured::{JsonSchemaConfig, JsonSchemaDefinition},
    models::tool::{Tool, FunctionDescription},
    Result,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure your API key is set.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Build the client.
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Define a tool the model can call.
    let weather_tool = Tool::Function {
        function: FunctionDescription {
            name: "get_current_weather".to_owned(),
            description: Some("Retrieve current weather for a given location".to_owned()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "Name of the city or location"
                    }
                },
                "required": ["location"]
            }),
        },
    };

    // Define a JSON Schema for structured output.
    let schema_def = JsonSchemaDefinition {
        schema_type: "object".to_owned(),
        properties: {
            let mut map = serde_json::Map::new();
            map.insert("result".to_owned(), json!({
                "type": "string",
                "description": "The answer generated by the model"
            }));
            map
        },
        required: Some(vec!["result".to_owned()]),
        additional_properties: Some(false),
    };

    let json_schema_config = JsonSchemaConfig {
        name: "answer".to_owned(),
        strict: true,
        schema: schema_def,
    };

    // Create a chat completion request with tool calling and structured output.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What is the meaning of life?".to_string(),
            name: None,
            tool_calls: None,
        }],
        stream: None,
        response_format: Some("json_schema".to_string()),
        // Attach the tool.
        tools: Some(vec![weather_tool]),
        // Other optional fields.
        provider: None,
        models: None,
        transforms: None,
    };

    // Build the request payload using the unified request builder (which supports structured outputs).
    let request_payload = client
        .completion_request(vec![Message {
            role: "user".to_string(),
            content: "What is the meaning of life?".to_string(),
            name: None,
            tool_calls: None,
        }])
        .with_structured_output(json_schema_config, true, false)
        .build();

    println!("Structured Request Payload:\n{}", serde_json::to_string_pretty(&request_payload)?);

    // Invoke the chat completion endpoint.
    let response = client.chat_completion(request).await?;
    println!("Response Model: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }

    Ok(())
}
```

### Running Tests

Before running tests, set the `OPENROUTER_API_KEY` environment variable to your API key:
```bash
export OPENROUTER_API_KEY=sk-...
cargo test
```
For verbose output:
```bash
cargo test -- --nocapture
```

## Implementation Plan

The project is under active development. The roadmap outlines upcoming features and milestones:

### Phase 1: Core Functionality (Completed/In Progress)
- [x] **Client Framework:**
  - Type‑state builder pattern for configuration with compile‑time validations.
  - Custom headers and robust error propagation.
- [x] **Chat Completion Endpoint:**
  - Synchronous chat completions with JSON decoding and streaming support.
- [x] **Core Data Models:**
  - Definitions for chat messages, requests, responses, and usage.

### Phase 2: Additional Endpoints and Features
- [x] **Streaming Support:**
  - Streaming API for chat completions via Server‑Sent Events (SSE).
- [x] **Web Search Endpoint:**
  - New endpoint for web search queries with strongly‑typed request/response models.
- [x] **Tool Calling & Structured Outputs:**
  - Support for invoking callable functions and validating structured responses via JSON Schema.
- [x] **Provider Preferences & Routing:**
  - Configuration options for model fallbacks, routing, and provider filtering.

### Phase 3: Robust Testing & Documentation
- [ ] **Test Coverage:**
  - Expand unit and integration tests, including streaming-specific tests.
- [ ] **Documentation Improvements:**
  - Enhance inline documentation, API docs, and usage examples in the `/examples` directory.
- [ ] **Continuous Integration (CI):**
  - Set up CI pipelines for automated builds and tests.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request with your ideas or fixes. Follow the code style guidelines and ensure that all tests pass.

## License

Distributed under either the MIT license or the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
