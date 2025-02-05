-------------------------------------------------
docs/Overview.md
-------------------------------------------------

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
│   │   └── parameters.rs      # LLM parameter types (stub for now)
│   ├── api/                   # API endpoint implementations
│   │   ├── mod.rs             # Exports for API implementations
│   │   ├── chat.rs            # Chat completion endpoint implementation
│   │   ├── completion.rs      # Text completion endpoint implementation
│   │   ├── generation.rs      # Generation metadata endpoint implementation
│   │   ├── models.rs          # Models listing endpoint implementation
│   │   └── credits.rs         # Credits information endpoint implementation
│   ├── types/                 # Shared type definitions (and stubs)
│   │   ├── mod.rs             # Exports for shared types
│   │   ├── provider.rs        # Provider-related types (stub)
│   │   ├── routing.rs         # Routing preference types (stub)
│   │   └── transform.rs       # Message transform types (stub)
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
│   └── streaming.rs         # Streaming example
├── Cargo.toml                # Package manifest and dependencies
├── README.md                 # High-level documentation and usage instructions
├── CHANGELOG.md              # Version history and release notes
└── LICENSE                   # License information (MIT or Apache-2.0)
```

---

## Key Design Considerations

1. **Modular Organization:**
   - **Models:** Contains all data structures and types representing API request/response schemas.
   - **API Modules:** Each API endpoint (chat, completions, etc.) is implemented in its own module.
   - **Types:** Common and shared types are defined separately to help reduce duplication.
   - **Utils:** Helper functions (HTTP, authentication, validation) are isolated.

2. **Clear Separation of Concerns:**
   The library separates endpoint implementations, data structures, and client configuration, improving maintainability and testability.

3. **Testing Structure:**
   - Integration tests are located in the `tests/` directory to ensure proper end-to-end behavior.
   - (Future) Advanced mock tests may be added in `tests/common/`.

4. **Documentation:**
   - The `examples/` directory includes sample usage for developers.
   - Inline Rust documentation (rustdoc) is provided throughout for clarity.
   - A CHANGELOG and LICENSE file are maintained.

5. **Robust Error Handling:**
   All error variants (including HTTP errors, API errors, configuration issues, and now structured output errors) are centralized in the error modules (`models/error.rs` and re-exported via `src/error.rs`). The library uses the `thiserror` crate to partition errors appropriately.

6. **Structured Outputs Support:**
   - **Overview:** Structured outputs allow the API to return responses adhering to a specified JSON Schema. This ensures that responses following tool calls and non‐interactive endpoints are consistent and type‑safe.
   - **Configuration:** Structured output support is integrated at the client level via the type‑state builder. Clients can enable structured output on any endpoint (except interactive chat) by invoking methods on the unified request builder.
   - **Validation:** Using the jsonschema crate along with serde, responses can be asynchronously validated against a strongly‑typed JSON Schema provided by the client.
   - **Error Handling:** If a model does not support structured outputs or if the JSON Schema validation fails, the client returns detailed error information via error variants like `StructuredOutputNotSupported` or `SchemaValidationError`. Optionally, a fallback to unstructured output can be enabled.

---

## Type‑State Builder Pattern

One of the core design features is the type‑state builder pattern for creating an `OpenRouterClient`. This pattern enforces that required configuration steps (like setting the base URL and API key) are performed at compile time before making API calls. For example:

- **Unconfigured State:**
  The client is initially created in an unconfigured state.

- **NoAuth State:**
  After setting the base URL (which must end with a trailing slash), the client transitions into a state that is not yet authenticated.

- **Ready State:**
  Once an API key is provided (and additional settings such as timeout, HTTP referer, and site title are set), the client transitions into the Ready state where HTTP resources (such as the reqwest client) are fully built. Only then can API methods like `chat_completion` be invoked.

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
    // Other API modules: completion, models, credits...
}
```

---

## Implementation Roadmap

Since the project is under active development, here is the current and planned implementation plan:

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
- [ ] **Tool Calling & Structured Outputs:**
  - Implement support for tool calls with JSON Schema validation. This includes integration of structured output support into the request builder with optional validation and fallback modes.
- [ ] **Provider Preferences & Routing:**
  - Add support for options such as model fallbacks, routing preferences, and provider filtering.

### Phase 3: Testing, Documentation, and CI
- [ ] **Test Coverage:**
  - Expand integration and unit tests.
  - Implement mock tests for development using a mock server.
- [ ] **Documentation:**
  - Enhance inline documentation.
  - Provide comprehensive examples in the `/examples` directory.
- [ ] **Continuous Integration:**
  - Set up CI pipelines for automated testing and linting.

---

## Summary

The OpenRouter API Client Library is designed to be a robust, modular, and type‑safe interface for interacting with the OpenRouter API. Its architecture uses a type‑state builder pattern to enforce proper configuration and resource initialization, ensuring that only a fully configured client can be used to make API calls.

With the new structured outputs support, clients can now request responses that adhere to a specified JSON Schema, ensuring consistency and type‑safety across non‑interactive endpoints. Detailed error handling and the option to validate or fallback on unstructured output further enhance the robustness of the SDK.

This document provides an overview of the directory structure, design decisions, and our phased implementation plan. As the project evolves, additional endpoints and features will be introduced following this modular and test‑driven design.

---


The next steps to fully complete and polish the feature integration include:

1. Documentation and README Updates
 • Update the SDK’s documentation to include details on how to configure tool calling (both in request payloads and response validation).
 • Add inline code comments and example usage in the docs to help consumers understand how to enable and use tool calling.

2. Extended Error Handling and Logging
 • Enhance the error messages for tool calls by including additional context (e.g., the failed function name and arguments).
 • Integrate structured logging or tracing so that production logs include detailed information during tool call validation failures.

3. Streaming Support (if applicable)
 • Evaluate whether tool calling responses need specialized handling in streaming mode.
 • If so, add support to parse and validate progressive tool call chunks.

4. Additional Unit and Integration Tests
 • Write more tests to cover various edge cases, such as multiple tool calls in a single message, concurrent tool call responses, and fallback behavior when JSON Schema validation fails.
 • Also create tests where the SDK recovers gracefully if the response includes partly-invalid tool call data and returns a fallback (if the consumer opted for that behavior).

5. Validate Structured Output with JSON Schema
 • Integrate and test asynchronous JSON Schema validation for structured outputs and tool call responses using the jsonschema crate.
 • Document the behavior if schema validation fails and fallback is enabled.

6. Code Cleanup & Refactoring
 • Perform a review of the new changes and ensure consistency with existing code conventions and type‑state patterns.
 • Refactor any common response handling logic into a dedicated module if needed for clarity.

7. Update CI/CD and Build Pipelines
 • Ensure that all new tests run successfully on your Continuous Integration pipeline.
 • Add any new linting rules related to the tool calling module to maintain code quality.
