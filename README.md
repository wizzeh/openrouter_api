Here's the updated README.md that includes information about the Model Context Protocol (MCP) client implementation:

# OpenRouter API Client Library

OpenRouter API Client Library is a Rust client for interfacing with the OpenRouter API. The library is designed to be modular, type‑safe, and intuitive. It uses a type‑state builder pattern for configuring and validating the client at compile time, ensuring that all required configuration (such as setting the base URL and API key) happens before attempting a request.

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
- **Model Context Protocol (MCP) Client:** Implements a JSON-RPC client for the [Model Context Protocol](https://modelcontextprotocol.io/), enabling seamless integration with MCP servers for enhanced context and tool access.

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
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variables
    let api_key = utils::load_api_key_from_env()?;

    // Build the client (Unconfigured -> NoAuth -> Ready)
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a minimal chat completion request
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

    // Invoke the chat completion endpoint
    let chat_api = client.chat()?;
    let response = chat_api.chat_completion(request).await?;

    // Output the model's response
    if let Some(choice) = response.choices.first() {
        println!("Chat Response: {}", choice.message.content);
    }
    Ok(())
}
```

#### Provider Preferences Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::models::provider_preferences::{DataCollection, ProviderPreferences, ProviderSort};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variables
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;
    
    // Create provider preferences
    let preferences = ProviderPreferences::new()
        .with_order(vec!["OpenAI".to_string(), "Anthropic".to_string()])
        .with_allow_fallbacks(true)
        .with_data_collection(DataCollection::Deny)
        .with_sort(ProviderSort::Throughput);
    
    // Create a request builder with provider preferences
    let request_builder = client.chat_request_builder(vec![
        Message {
            role: "user".to_string(),
            content: "Hello with provider preferences!".to_string(),
            name: None,
            tool_calls: None,
        },
    ]);
    
    // Add provider preferences and build the payload
    let payload = request_builder
        .with_provider_preferences(preferences)?
        .build();
    
    // The payload now includes provider preferences!
    println!("Request payload: {}", serde_json::to_string_pretty(&payload)?);
    
    Ok(())
}
```

#### Model Context Protocol (MCP) Client Example

```rust
use openrouter_api::{MCPClient, Result};
use openrouter_api::mcp_types::{
    ClientCapabilities, GetResourceParams, ToolCallParams,
    MCP_PROTOCOL_VERSION
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new MCP client
    let client = MCPClient::new("https://mcp-server.example.com/mcp")?;
    
    // Initialize the client with client capabilities
    let server_capabilities = client.initialize(ClientCapabilities {
        protocolVersion: MCP_PROTOCOL_VERSION.to_string(),
        supportsSampling: Some(true),
    }).await?;
    
    println!("Connected to MCP server with capabilities: {:?}", server_capabilities);
    
    // Get a resource from the MCP server
    let resource = client.get_resource(GetResourceParams {
        id: "document-123".to_string(),
        parameters: None,
    }).await?;
    
    println!("Retrieved resource: {}", resource.content);
    
    // Call a tool on the MCP server
    let result = client.tool_call(ToolCallParams {
        id: "search-tool".to_string(),
        parameters: serde_json::json!({
            "query": "Rust programming"
        }),
    }).await?;
    
    println!("Tool call result: {:?}", result.result);
    
    Ok(())
}
```

#### Text Completion Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::completion::CompletionRequest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a text completion request
    let request = CompletionRequest {
        model: "openai/gpt-3.5-turbo-instruct".to_string(),
        prompt: "Once upon a time".to_string(),
        // Additional generation parameters
        extra_params: json!({
            "temperature": 0.8,
            "max_tokens": 50
        }),
    };

    // Invoke the text completion endpoint
    let completions_api = client.completions()?;
    let response = completions_api.text_completion(request).await?;

    // Print out the generated text
    if let Some(choice) = response.choices.first() {
        println!("Text Completion: {}", choice.text);
    }
    Ok(())
}
```

#### Streaming Chat Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use futures::StreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a chat completion request with streaming enabled
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

    // Invoke the streaming chat completion endpoint
    let chat_api = client.chat()?;
    let mut stream = chat_api.chat_completion_stream(request);

    // Process the stream
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if let Some(choice) = c.choices.first() {
                    print!("{}", choice.message.content);
                    std::io::stdout().flush().unwrap();
                }
            },
            Err(e) => eprintln!("Error during streaming: {}", e),
        }
    }
    println!();
    Ok(())
}
```

## Model Context Protocol (MCP) Client

The library includes a client implementation for the [Model Context Protocol](https://modelcontextprotocol.io/), which is an open protocol that standardizes how applications provide context to LLMs.

Key features of the MCP client include:

- **JSON-RPC Communication:** Implements the JSON-RPC 2.0 protocol for MCP
- **Resource Access:** Retrieve resources from MCP servers
- **Tool Invocation:** Call tools provided by MCP servers
- **Prompt Execution:** Execute prompts on MCP servers
- **Server Capabilities:** Discover and leverage server capabilities
- **Proper Authentication:** Handle initialization and authentication flows

```rust
// Create an MCP client connected to a server
let client = MCPClient::new("https://mcp-server.example.com/mcp")?;

// Initialize with client capabilities
let server_capabilities = client.initialize(ClientCapabilities {
    protocolVersion: "2025-03-26".to_string(),
    supportsSampling: Some(true),
}).await?;

// Access resources from the server
let resource = client.get_resource(GetResourceParams {
    id: "some-resource-id".to_string(),
    parameters: None,
}).await?;
```

See the [Model Context Protocol specification](https://spec.modelcontextprotocol.io/specification/2025-03-26/) for more details.

## Implementation Plan

The project is actively developed with the following roadmap:

### Phase 1: Core Functionality (Completed)
- [x] **Client Framework:**
  - Type‑state builder pattern for configuration with compile‑time validations.
  - Custom headers and robust error propagation.
- [x] **Chat Completion Endpoint:**
  - Synchronous chat completions with JSON decoding and streaming support.
- [x] **Core Data Models:**
  - Definitions for chat messages, requests, responses, and usage.

### Phase 2: Additional Endpoints and Features (Completed/In Progress)
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
- [x] **Model Context Protocol (MCP) Client:**
  - Client implementation for the standardized MCP protocol.
- [ ] **Models Listing and Credits:**
  - Implement endpoints to list models and fetch credit details.

### Phase 3: Robust Testing & Documentation (In Progress)
- [ ] **Test Coverage:**
  - Expand unit and integration tests, including MCP and streaming-specific tests.
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

_**Version:** 0.1.3 • **License:** MIT / Apache‑2.0_

The `openrouter_api` crate is a comprehensive client for interacting with the [OpenRouter API](https://openrouter.ai/docs) and [Model Context Protocol](https://modelcontextprotocol.io/) servers. It provides strongly‑typed endpoints for chat completions, text completions, web search, and MCP connections. The crate is built using asynchronous Rust and leverages advanced patterns for safe and flexible API usage.

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
  - [Model Context Protocol](#model-context-protocol)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Additional Resources](#additional-resources)

---

## Core Concepts

- **Type‑State Client Configuration:**
  The client is built using a type‑state pattern to ensure that required parameters are set before making any API calls.

- **Provider Preferences:**
  Strongly-typed configuration for model routing, fallbacks, and provider selection.

- **Asynchronous Streaming:**
  Support for streaming responses via asynchronous streams.

- **Model Context Protocol:**
  Client implementation for connecting to MCP servers to access resources, tools, and prompts.

- **Error Handling & Validation:**
  Comprehensive error handling with detailed context and validation utilities.

---

## Architecture & Module Overview

The crate is organized into several modules:

- **`client`:** Type-state client implementation with builder pattern
- **`api`:** API endpoint implementations (chat, completions, web search, etc.)
- **`models`:** Domain models for structured outputs, provider preferences, tools
- **`types`:** Type definitions for requests and responses
- **`mcp`:** Model Context Protocol client implementation
- **`error`:** Centralized error handling
- **`utils`:** Utility functions and helpers

---

## Client Setup & Type‑State Pattern

```rust
// Create an unconfigured client
let client = OpenRouterClient::new()
    // Transition to NoAuth state by setting base URL
    .with_base_url("https://openrouter.ai/api/v1/")?
    .with_timeout(Duration::from_secs(30))
    .with_http_referer("https://your-app.com/")
    // Transition to Ready state by setting API key
    .with_api_key(std::env::var("OPENROUTER_API_KEY")?)?;
```

## API Endpoints

### Chat Completions

```rust
// Basic chat completion
let response = client.chat()?.chat_completion(
    ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Explain quantum computing".to_string(),
            name: None,
            tool_calls: None,
        }],
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    }
).await?;
```

### Tool Calling

```rust
// Define a function tool
let weather_tool = Tool::Function { 
    function: FunctionDescription {
        name: "get_weather".to_string(),
        description: Some("Get weather information for a location".to_string()),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City and state"
                }
            },
            "required": ["location"]
        }),
    }
};

// Make a request with tool calling enabled
let response = client.chat()?.chat_completion(
    ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What's the weather in Boston?".to_string(),
            name: None,
            tool_calls: None,
        }],
        tools: Some(vec![weather_tool]),
        // other fields...
        stream: None,
        response_format: None,
        provider: None,
        models: None,
        transforms: None,
    }
).await?;
```

### Model Context Protocol

```rust
// Create an MCP client
let mcp_client = MCPClient::new("https://mcp-server.example.com/mcp")?;

// Initialize with client capabilities
let server_capabilities = mcp_client.initialize(ClientCapabilities {
    protocolVersion: MCP_PROTOCOL_VERSION.to_string(),
    supportsSampling: Some(true),
}).await?;

// Access a resource from the MCP server
let resource = mcp_client.get_resource(GetResourceParams {
    id: "document-123".to_string(),
    parameters: None,
}).await?;
```

## Error Handling

```rust
match client.chat()?.chat_completion(request).await {
    Ok(response) => {
        println!("Success: {}", response.choices[0].message.content);
    },
    Err(e) => match e {
        Error::ApiError { code, message, .. } => {
            eprintln!("API Error ({}): {}", code, message);
        },
        Error::HttpError(ref err) if err.is_timeout() => {
            eprintln!("Request timed out!");
        },
        Error::ConfigError(msg) => {
            eprintln!("Configuration error: {}", msg);
        },
        _ => eprintln!("Other error: {:?}", e),
    }
}
```

## Best Practices

1. **Use the Type‑State Pattern:**
   Let the compiler ensure your client is properly configured.

2. **Set Appropriate Timeouts & Headers:**
   Configure reasonable timeouts and identify your application.

3. **Handle Errors Appropriately:**
   Implement proper error handling for each error type.

4. **Use Provider Preferences:**
   Configure provider routing for optimal model selection.

5. **Secure Your API Keys:**
   Store keys in environment variables or secure storage.

## Additional Resources

- [OpenRouter API Documentation](https://openrouter.ai/docs)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/2025-03-26/)

---
