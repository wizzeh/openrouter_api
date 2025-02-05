------------------------------------------------
# OpenRouter API Client Library Overview

This project is a Rust client library for the OpenRouter API that matches the documentation and follows Rust best practices. It is designed to be modular, maintainable, and type‑safe; it uses a type‑state builder pattern for client configuration so that required settings (such as the base URL and API key) are enforced at compile time.

> **Note:** This project is still in active development. Many features are planned and under iterative improvement.

---

## Project Directory Structure

The project is organized as follows:

```
openrouter_api/
├── src/
│   ├── lib.rs                 # Library root, public exports, and high-level documentation
│   ├── client.rs              # OpenRouter client implementation and type‑state builder
│   ├── config.rs              # (Future) Dedicated client configuration module (currently merged into client.rs)
│   ├── models/                # Data models and types for API endpoints
│   │   ├── mod.rs             # Exports for models
│   │   ├── chat.rs            # Chat completion types
│   │   ├── completion.rs      # Text completion types
│   │   ├── common.rs          # Shared types used across endpoints
│   │   ├── error.rs           # API error types and conversions
│   │   ├── parameters.rs      # LLM parameter types (stub for now)
│   │   └── provider_preferences.rs  # Provider preferences & routing settings
│   ├── api/                   # API endpoint implementations
│   │   ├── mod.rs             # Exports for API implementations
│   │   ├── chat.rs            # Chat completion endpoint implementation
│   │   ├── completion.rs      # Text completion endpoint implementation
│   │   ├── generation.rs      # Generation metadata endpoint implementation
│   │   ├── models.rs          # Models listing endpoint implementation
│   │   ├── credits.rs         # Credits information endpoint implementation
│   │   └── web_search.rs      # Web search endpoint implementation (new)
│   ├── types/                 # Shared type definitions (and stubs)
│   │   ├── mod.rs             # Exports for shared types
│   │   ├── provider.rs        # Provider-related types (stub)
│   │   ├── routing.rs         # Routing preference types (stub)
│   │   ├── transform.rs       # Message transform types (stub)
│   │   └── web_search.rs      # Web search request and response types (new)
│   └── utils/                 # Utility functions
│       ├── mod.rs             # Exports for utilities
│       ├── http.rs            # HTTP-related utility functions
│       ├── auth.rs            # Authentication utilities (if needed)
│       └── validation.rs      # Input validation utilities
├── tests/                     # Integration tests
│   ├── common/               # Shared test utilities (for mocking, etc.)
│   │   ├── mod.rs             # Exports for test utilities
│   │   └── mock_server.rs     # Mock server for testing (to be added later)
│   ├── chat_tests.rs         # Chat completion endpoint tests
│   ├── completion_tests.rs   # Text completion endpoint tests
│   └── integration_tests.rs  # Full integration tests
├── examples/                  # Usage examples and sample applications
│   ├── chat.rs              # Simple chat completion example
│   ├── completion.rs        # Text completion example
│   ├── streaming.rs         # Streaming example
│   └── minimal_chat.rs      # Minimal chat example (new)
├── Cargo.toml                # Package manifest and dependencies
├── README.md                 # High-level documentation and usage instructions
├── CHANGELOG.md              # Version history and release notes
└── LICENSE                   # License information (MIT or Apache-2.0)
```

---

## Key Design Considerations

1. **Modular Organization:**
   - **Models:** Contains all data structures and types representing API request/response schemas. The recently added strongly‑typed `ProviderPreferences` and `WebSearch` modules allow users to configure routing options and perform web search queries with enhanced type‑safety.
   - **API Modules:** Each API endpoint (chat, completions, web_search, etc.) is implemented in its own module.
   - **Types:** Common and shared types are defined separately to help reduce duplication.
   - **Utils:** Helper functions (HTTP, authentication, validation) are isolated.

2. **Clear Separation of Concerns:**
   The library separates endpoint implementations, data structures, and client configuration, thereby improving maintainability and testability.

3. **Testing Structure:**
   - Integration tests are located in the `tests/` directory to ensure proper end-to-end behavior.
   - (Future) Advanced mock tests may be added in `tests/common/`.

4. **Documentation:**
   - The `examples/` directory includes sample usage for developers.
   - Inline Rust documentation (rustdoc) is provided throughout for clarity.
   - A CHANGELOG and LICENSE file are maintained.

5. **Robust Error Handling:**
   All error variants (including HTTP errors, API errors, configuration issues, structured output errors, and provider preference errors) are centralized in the error modules (`models/error.rs` and re-exported via `src/error.rs`). The library uses the `thiserror` crate to partition errors appropriately.

6. **Structured Outputs Support:**
   - **Overview:** Structured outputs allow the API to return responses adhering to a specified JSON Schema. This ensures that responses following tool calls and non‑interactive endpoints are consistent and type‑safe.
   - **Configuration:** Structured output support is integrated at the client level via the type‑state builder. Clients can enable structured output on any endpoint (except interactive chat) by invoking methods on the unified request builder.
   - **Validation:** Using the `jsonschema` crate along with `serde`, responses can be asynchronously validated against a strongly‑typed JSON Schema provided by the client.
   - **Error Handling:** If a model does not support structured outputs or if JSON Schema validation fails, the client returns detailed error information via variants like `StructuredOutputNotSupported` or `SchemaValidationError`. A fallback to unstructured output can also be enabled if desired.

7. **Provider Preferences & Routing:**
   - **Overview:** A new feature that enables developers to configure routing preferences for model calls via a strongly‑typed `ProviderPreferences` interface. This includes options such as provider ordering, fallback behavior, parameter requirements, data collection policies, providers to ignore, quantization filters, and sorting strategies.
   - **Implementation:** These preferences are integrated into the unified request builder as additional parameters. They are serialized into the `provider` field of the request payload to be handled by the API, while errors relating to misconfigurations are centrally managed.
   - **Extensibility:** While the current implementation focuses on the happy path, the design supports future runtime and (potentially) compile-time validations as provider capabilities evolve.

8. **New Web Search Endpoint:**
   - **Overview:** In addition to chat and text completion endpoints, a new web search endpoint has been introduced. This endpoint accepts a basic search query (with an optional result count) and returns structured search results.
   - **Types & API:** The `WebSearchRequest` and `WebSearchResponse` types are defined under `src/types/web_search.rs`, and the implementation lives in `src/api/web_search.rs`. This approach maintains the library’s modular design and type safety.

---

## Type‑State Builder Pattern

One of the core design features is the type‑state builder pattern for creating an `OpenRouterClient`. This pattern enforces that required configuration steps (like setting the base URL and API key) are performed at compile time before making API calls. For example:

- **Unconfigured State:**
  The client is initially created in an unconfigured state.

- **NoAuth State:**
  After setting the base URL (which must end with a trailing slash), the client transitions into a state that is not yet authenticated.

- **Ready State:**
  Once an API key is provided (and additional settings such as timeout, HTTP referer, and site title are set), the client transitions into the Ready state where HTTP resources (such as the reqwest client) are fully built. Only then can API methods like `chat_completion` or `web_search` be invoked.

The type definitions for the state markers (`Unconfigured`, `NoAuth`, and `Ready`) are defined in `src/client.rs` and re-exported through `src/lib.rs`.

Below is an excerpt of the type‑state builder implementation for clarity:

```rust
// src/client.rs (excerpt)

use std::marker::PhantomData;
use std::time::Duration;
use url::Url;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use crate::error::{Error, Result};
use crate::types;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn build_headers(&self) -> HeaderMap { /* ... */ }
}

// State markers.
pub struct Unconfigured;
pub struct NoAuth;
pub struct Ready;

/// Main OpenRouter client using type‑state validation.
pub struct OpenRouterClient<State = Unconfigured> {
    pub config: ClientConfig,
    pub http_client: Option<reqwest::Client>,
    pub _state: PhantomData<State>,
}

impl OpenRouterClient<Unconfigured> {
    // new(), with_base_url(), and transitioning methods...
}

impl OpenRouterClient<NoAuth> {
    // with_api_key(), with_timeout(), etc. transitioning to Ready...
}

impl OpenRouterClient<Ready> {
    /// Access the chat API.
    pub fn chat(&self) -> chat::ChatApi {
        chat::ChatApi::new(self.http_client.clone().unwrap(), &self.config)
    }
    // Other API modules: completion, web_search, models, credits, etc.
}
```

---

## Minimal Code Examples

### Minimal Chat Example

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
        .with_api_key(api_key);

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

### Minimal Web Search Example

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
        .with_api_key(api_key);

    // Create a minimal web search request.
    let request = WebSearchRequest {
        query: "rust programming".into(),
        num_results: Some(5),
    };

    // Invoke the web search endpoint.
    let response: WebSearchResponse = client.web_search().search(request).await?;

    // Print out the search results.
    println!("Search query: {}", response.query);
    for result in response.results {
        println!("Title: {}\nURL: {}\n", result.title, result.url);
    }

    Ok(())
}
```

---

## Implementation Roadmap

### Phase 1: Core Functionality (In Progress / Completed)
- [x] **Client Framework:**
  - Implement type‑state builder pattern for configuration.
  - Validate required fields at compile time.
- [x] **Chat Completion Endpoint:**
  - Implement `chat_completion` to call the OpenRouter chat completions API.
  - Provide basic error handling and JSON decoding.
- [x] **Core Data Models:**
  - Define types for chat messages, requests, responses, and usage.
  - Implement error types for consistent error handling.

### Phase 2: Additional Endpoints and Advanced Features
- [ ] **Streaming Support:**
  - Implement a streaming API for chat completions.
- [ ] **Text Completion Endpoint:**
  - Build similar functionality for text completions.
- [ ] **Models Listing and Credits:**
  - Implement endpoints to list models and fetch credit details.
- [x] **Tool Calling & Structured Outputs:**
  - Implement support for tool calls with JSON Schema validation, including optional validation and fallback modes.
- [x] **Provider Preferences & Model Routing:**
  - Enable configuration for provider preferences, fallbacks, and routing options via the strongly‑typed ProviderPreferences module.
- [x] **Web Search Endpoint:**
  - Introduce a new endpoint for web search queries with type‑safe request and response models.

### Phase 3: Robust Testing & Documentation
- [ ] **Test Coverage:**
  - Expand integration and unit tests.
  - Implement mock tests using a dedicated mock server.
- [ ] **Documentation Improvements:**
  - Enhance inline documentation and API docs using rustdoc.
  - Provide additional usage examples in the `/examples` directory.
- [ ] **Continuous Integration (CI):**
  - Set up CI pipelines for continuous builds and testing.

---

## Summary

The OpenRouter API Client Library is designed as a robust, modular, and type‑safe interface for interacting with the OpenRouter API. Its architecture hinges on a type‑state builder pattern that enforces proper client configuration and resource initialization, guaranteeing that only a fully configured client can perform API calls.

With the recent introduction of structured outputs, provider preferences, and the new web search endpoint, clients can now obtain responses that adhere to a specified JSON Schema, route requests based on strongly‑typed provider settings, and perform web searches seamlessly. Detailed error handling and strict configuration checks further contribute to the library’s reliability and ease of use.

As the project continues to evolve, additional endpoints, features, and tests will be added following this modular and test‑driven design.

------------------------------------------------
