# OpenRouter API Client Library

OpenRouter API Client Library is a Rust client for interfacing with the OpenRouter API. The library is designed to be modular, type‑safe, and intuitive. It uses a type‑state builder pattern for configuring and validating the client at compile time, ensuring that all required configuration (such as setting the base URL and API key) happens before attempting a request.

> **Note:** This project is still in development. Many features are planned but not yet fully implemented.

## Features

- **Modular Organization:** Organized into clear modules for models, API endpoints, common types, and utilities.
- **Type‑State Builder:** Guarantees compile‑time validation of client configuration (e.g. base URL, API key, custom headers) for a robust development experience.
- **HTTP Integration:** Uses [reqwest](https://crates.io/crates/reqwest) with rustls‑tls for secure asynchronous HTTP requests.
- **Robust Error Handling:** Centralized error management using the `thiserror` crate ensures consistent error types across the library.
- **Streaming Support:** Supports streaming chat completions via Server‑Sent Events (SSE). The library gracefully skips over comment lines and non‑JSON payloads, letting you update UIs in real‑time.
- **Structured Outputs:** Optionally request structured responses with JSON Schema validation so that responses strictly follow your defined schema.
- **Tool Calling Capability:** Define function‑type tools that the model can invoke. Supports concurrent tool calls in a single response with proper validation against expected formats.
- **Provider Preferences & Routing:** Configure model fallbacks, routing preferences, and provider filtering via a strongly‑typed interface.
- **Web Search Endpoint:** Easily perform web search queries with type‑safe request and response models.
- **Text Completion Endpoint:** Send a prompt (with a required `model` and `prompt` field) and receive generated text completions along with additional generation details. Extra parameters (e.g. temperature, top_p, etc.) can be provided as needed.

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```bash
cargo add openrouter_api
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

#### Minimal Text Completion Example

```rust
use openrouter_api::{OpenRouterClient, Ready, Result};
use openrouter_api::types::completion::{CompletionRequest, CompletionResponse};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure your API key is set in the environment.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Build the client (Unconfigured -> NoAuth -> Ready).
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a minimal text completion request.
    let request = CompletionRequest {
        model: "model".to_string(),
        prompt: "Once upon a time".to_string(),
        // Additional generation parameters can be set here.
        extra_params: json!({
            "temperature": 0.8,
            "max_tokens": 50
        }),
    };

    // Invoke the text completion endpoint.
    let response: CompletionResponse = client.completions().text_completion(request).await?;

    // Print out the generated text from the first choice.
    if let Some(choice) = response.choices.first() {
        println!("Text Completion: {}", choice.text);
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
- [x] **Text Completion Endpoint:**
  - New endpoint for text completions, accepting a prompt and returning generated text along with extra details.
- [x] **Tool Calling & Structured Outputs:**
  - Support for invoking callable functions and validating structured responses via JSON Schema.
- [x] **Provider Preferences & Routing:**
  - Configuration options for model fallbacks, routing, and provider filtering.
- [ ] **Models Listing and Credits:**
    - Implement endpoints to list models and fetch credit details.

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


---

# OpenRouter API Rust Crate Documentation

_**Version:** 0.1.2 • **License:** MIT / Apache‑2.0_

The `openrouter_api` crate is a comprehensive client for interacting with the [OpenRouter API](https://openrouter.ai/docs). It provides strongly‑typed endpoints for chat completions, text completions, web search, and more. The crate is built using asynchronous Rust (with [reqwest](https://docs.rs/reqwest/) and [tokio](https://tokio.rs/)) and leverages advanced patterns such as type‑state and builder patterns for safe and flexible API usage.

---

## Table of Contents

- [Core Concepts](#core-concepts)
- [Installation](#installation)
- [Architecture & Module Overview](#architecture--module-overview)
- [Client Setup & Type‑State Pattern](#client-setup--type-state-pattern)
- [API Endpoints](#api-endpoints)
  - [Chat Completions](#chat-completions)
  - [Text Completions](#text-completions)
  - [Web Search](#web-search)
  - [Tool Calling & Structured Output](#tool-calling--structured-output)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Additional Resources](#additional-resources)

---

## Core Concepts

- **Type‑State Client Configuration:**
  The client is built using a type‑state pattern to ensure that required parameters (e.g. API key and base URL) are set before making any API calls. This pattern leverages Rust’s type system (with [`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)) to prevent misconfiguration at compile time.

- **Flexible Request Building:**
  The `RequestBuilder` lets you build rich API requests including optional provider preferences, structured output (using JSON Schema), and tool calling configurations.

- **Asynchronous Streaming:**
  The chat API supports streaming responses via asynchronous streams. This is implemented using crates like [async-stream](https://docs.rs/async-stream/) and [tokio-util](https://docs.rs/tokio-util/).

- **Error Handling & Schema Validation:**
  A comprehensive custom error type wraps HTTP errors, API errors, configuration mistakes, and JSON Schema validation issues.

- **Tool Calling:**
  Easily integrate with external functions using a structured representation of function calls. This enables the API to suggest or even invoke tools based on the conversation context.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
openrouter_api = "0.1.2"
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-stream = "0.3"
tokio-util = "0.7"
thiserror = "1.0"
```

---

## Architecture & Module Overview

The crate is organized into several modules:

- **`client`:**
  Contains the client configuration and type‑state builder implementation. It ensures proper initialization before any API calls are made.

- **`api`:**
  Provides submodules for each API endpoint (chat, completion, web search, etc.). Each submodule includes detailed implementations for constructing requests and handling responses.

- **`models`:**
  Defines domain models for structured outputs, provider preferences, and tool calling.

- **`types`:**
  Contains type definitions for chat messages, completions, and web search responses.

- **`error`:**
  Centralized error types (wrapping reqwest errors, API errors, configuration issues, and schema validation problems).

- **`tests`:**
  Integration tests showcasing how to simulate and validate API responses.

---

## Client Setup & Type‑State Pattern

The client configuration follows a three‑step type‑state builder pattern:

1. **Unconfigured:**
   The client is created in an unconfigured state using `OpenRouterClient::new()`.
2. **NoAuth:**
   The base URL is set (via `.with_base_url()`), transitioning the client into a state where authentication is not yet provided.
3. **Ready:**
   The API key is added (via `.with_api_key()`), and the client becomes fully configured for making API calls.

### Example: Configuring the Client

```rust
use openrouter_api::client::{OpenRouterClient, Unconfigured};
use std::time::Duration;

// Create an unconfigured client.
let client = OpenRouterClient::<Unconfigured>::new();

// Transition to a configured client (NoAuth state) by setting the base URL.
let client = client
    .with_base_url("https://openrouter.ai/api/v1/")?
    .with_timeout(Duration::from_secs(30))
    .with_http_referer("https://your-app.com/")
    .with_site_title("Your App Name");

// Supply the API key to transition into the Ready state.
let client = client.with_api_key(std::env::var("OPENROUTER_API_KEY")?)?;
```

> **Note:** The type‑state pattern prevents you from accidentally making API calls without proper configuration. Attempting to call an endpoint on a client that isn’t in the `Ready` state will result in a compile‑time error.

---

## API Endpoints

The crate supports multiple endpoints, each with its own module and specialized request/response types.

### Chat Completions

The chat API supports both single‑shot and streaming completions.

#### Single‑Shot Chat Completion

```rust
use openrouter_api::types::chat::Message;

let messages = vec![Message {
    role: "user".to_string(),
    content: "Explain quantum computing".to_string(),
    name: None,
    tool_calls: None,
}];

// Issue a single chat completion call.
let chat_api = client.chat()?;
let response = chat_api.chat_completion(
    openrouter_api::types::chat::ChatCompletionRequest {
        model: "mistralai/mistral-small-latest".to_string(),
        messages: messages.clone(),
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    }
).await?;

println!("Assistant: {}", response.choices[0].message.content);
```

#### Streaming Chat Completion

Streaming is useful for real‑time applications. The stream returns chunks as they become available.

```rust
use futures::StreamExt;

let chat_api = client.chat()?;
let mut stream = chat_api.chat_completion_stream(
    openrouter_api::types::chat::ChatCompletionRequest {
        model: "mistralai/mistral-small-latest".to_string(),
        messages: messages.clone(),
        stream: Some(true),
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    }
);

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(c) => {
            // Each chunk may include partial content updates.
            if let Some(choice) = c.choices.first() {
                // Print the delta (partial content) if available.
                print!("{}", choice.message.content);
            }
        }
        Err(e) => {
            eprintln!("Stream error: {:?}", e);
            break;
        }
    }
}
```

### Text Completions

For pure text completions, the `CompletionApi` is used.

```rust
use openrouter_api::types::completion::CompletionRequest;

let request = CompletionRequest {
    model: "openai/gpt-4".to_string(),
    prompt: "Once upon a time".to_string(),
    extra_params: serde_json::json!({ "temperature": 0.7 }),
};

let completion_api = openrouter_api::api::completion::CompletionApi::new(
    client.http_client.clone().unwrap(),  // Clone the reqwest client
    &client.config,
);

let response = completion_api.text_completion(request).await?;
println!("Completed Text: {}", response.choices[0].text);
```

### Web Search

The web search API allows you to perform simple search queries and receive structured results.

```rust
use openrouter_api::types::web_search::WebSearchRequest;

let search_request = WebSearchRequest {
    query: "rust programming".to_string(),
    num_results: Some(5),
};

let web_search_api = client.web_search()?;
let search_response = web_search_api.search(search_request).await?;

println!("Query: {}", search_response.query);
for result in search_response.results {
    println!("Title: {} • URL: {}", result.title, result.url);
}
```

### Tool Calling & Structured Output

#### Enabling Structured Output

Structured output allows you to validate the model’s response against a JSON Schema. This is configured via the `RequestBuilder`.

```rust
use openrouter_api::models::structured::JsonSchemaConfig;
use openrouter_api::api::request::RequestBuilder;

let schema_config = JsonSchemaConfig {
    name: "Person".to_string(),
    strict: true,
    schema: serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    }),
};

let request_payload = RequestBuilder::new("mistralai/mistral-small-latest", messages.clone(), serde_json::json!({}))
    .with_structured_output(schema_config, true, true)  // validate & fallback options
    .build();
```

#### Configuring Tool Calling

Tool calling lets you define external functions that the model can suggest invoking. This is useful for integrating live functions (e.g., fetching weather).

```rust
use openrouter_api::models::tool::{Tool, FunctionDescription};

let get_weather = FunctionDescription {
    name: "get_weather".to_string(),
    description: Some("Get weather information for a location".to_string()),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City and state (e.g., 'San Francisco, CA')"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    }),
};

let request_payload = RequestBuilder::new("meta-llama/llama-3-70b-instruct", messages.clone(), serde_json::json!({}))
    .build();

// Call the chat endpoint with tool information.
let response = client.chat()?.chat_completion(
    openrouter_api::types::chat::ChatCompletionRequest {
        model: "meta-llama/llama-3-70b-instruct".to_string(),
        messages,
        stream: None,
        response_format: None,
        tools: Some(vec![Tool::Function { function: get_weather }]),
        provider: None,
        models: None,
        transforms: None,
    }
).await?;

// Process the tool call response.
if let Some(choice) = response.choices.first() {
    if let Some(tool_calls) = &choice.message.tool_calls {
        for call in tool_calls {
            if call.kind == "function" {
                println!("Tool call detected for function: {}", call.function_call.name);
                println!("Arguments: {}", call.function_call.arguments);
                // Here, you would implement your logic to execute the tool and return the result.
            }
        }
    }
}
```

---

## Error Handling

The crate defines a central [`Error`](#error-handling) enum that wraps different error kinds:

- **HTTP Errors:**
  Errors coming from the reqwest library.

- **API Errors:**
  Errors returned from the OpenRouter API (with status codes and error messages).

- **Configuration Errors:**
  Issues with client setup (e.g., invalid API key or headers).

- **Schema Validation Errors:**
  When a response fails JSON Schema validation for structured output.

### Example Error Handling

```rust
match client.chat()?.chat_completion(
    openrouter_api::types::chat::ChatCompletionRequest {
        model: "openai/gpt-4".to_string(),
        messages: messages.clone(),
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    }
).await {
    Ok(response) => {
        println!("Chat completion succeeded: {:?}", response);
    },
    Err(e) => match e {
        openrouter_api::error::Error::ApiError { code, message, .. } => {
            eprintln!("API Error ({}): {}", code, message);
        },
        openrouter_api::error::Error::HttpError(ref err) if err.is_timeout() => {
            eprintln!("Request timed out!");
        },
        _ => eprintln!("Unexpected error: {:?}", e),
    },
}
```

---

## Best Practices

1. **Use the Type‑State Pattern:**
   Always configure your client fully (base URL, API key, headers) before issuing requests. The type‑state pattern will help catch misconfiguration at compile time.

2. **Set Appropriate Timeouts & Headers:**
   Leverage `with_timeout`, `with_http_referer`, and `with_site_title` to ensure your requests are traceable and robust.

3. **Validate Structured Responses:**
   When using structured output, configure JSON Schema validation to catch unexpected responses early.

4. **Stream Responsibly:**
   When using streaming endpoints, ensure you handle errors gracefully and close streams when finished.

5. **Secure Your API Keys:**
   Store sensitive API keys in environment variables or secure storage rather than hardcoding them.

---

## Examples

### Full Client Initialization & Chat Request

Below is a complete example combining client initialization, a chat request, and error handling:

```rust
use openrouter_api::client::{OpenRouterClient, Unconfigured};
use openrouter_api::types::chat::{Message, ChatCompletionRequest};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client (Unconfigured -> NoAuth -> Ready)
    let client = OpenRouterClient::<Unconfigured>::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_timeout(Duration::from_secs(30))
        .with_http_referer("https://your-app.com/")
        .with_site_title("Example App")
        .with_api_key(std::env::var("OPENROUTER_API_KEY")?)?;

    // Prepare chat messages.
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "What is a phantom type in Rust?".to_string(),
            name: None,
            tool_calls: None,
        }
    ];

    // Build the chat completion request.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4".to_string(),
        messages,
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    // Execute the chat completion.
    let chat_api = client.chat()?;
    match chat_api.chat_completion(request).await {
        Ok(response) => {
            println!("Assistant says: {}", response.choices[0].message.content);
        },
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }

    Ok(())
}
```

### Provider Preferences with Request Builder

```rust
use openrouter_api::api::request::RequestBuilder;
use serde_json::json;

let provider_preferences = json!({
    "order": ["openrouter"],
    "allowFallbacks": true
});

let request_payload = RequestBuilder::new("openai/gpt-4", messages.clone(), json!({}))
    .with_provider_preferences(provider_preferences, true, true)
    .build();

println!("Payload: {}", serde_json::to_string_pretty(&request_payload)?);
```

---

## Additional Resources

- [OpenRouter API Documentation](https://openrouter.ai/docs)

---
