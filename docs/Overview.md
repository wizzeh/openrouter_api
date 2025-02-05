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
   - **Models:** Contains all data structures and types representing API request/response schemas. The recent addition of a strongly‑typed `ProviderPreferences` module allows users to configure routing options—including provider ordering, fallback behavior, and quantization filtering—with enhanced type‑safety.
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
   All error variants (including HTTP errors, API errors, configuration issues, structured output errors, and now provider preference errors) are centralized in the error modules (`models/error.rs` and re-exported via `src/error.rs`). The library uses the `thiserror` crate to partition errors appropriately.

6. **Structured Outputs Support:**
   - **Overview:** Structured outputs allow the API to return responses adhering to a specified JSON Schema. This ensures that responses following tool calls and non‑interactive endpoints are consistent and type‑safe.
   - **Configuration:** Structured output support is integrated at the client level via the type‑state builder. Clients can enable structured output on any endpoint (except interactive chat) by invoking methods on the unified request builder.
   - **Validation:** Using the jsonschema crate along with serde, responses can be asynchronously validated against a strongly‑typed JSON Schema provided by the client.
   - **Error Handling:** If a model does not support structured outputs or if JSON Schema validation fails, the client returns detailed error information via error variants like `StructuredOutputNotSupported` or `SchemaValidationError`. Optionally, a fallback to unstructured output can be enabled.

7. **Provider Preferences & Routing:**
   - **Overview:** A new feature that enables developers to configure routing preferences for model calls via a strongly‑typed `ProviderPreferences` interface. This includes options such as provider ordering, fallback behavior, parameter requirements, data collection policies, providers to ignore, quantization filters, and sorting strategies.
   - **Implementation:** These preferences are integrated into the unified request builder as additional parameters. They are serialized into the `provider` field of the request payload to be handled by the API, while errors relating to misconfigurations are centrally managed.
   - **Extensibility:** While the current implementation focuses on the happy path, the design supports future runtime and (potentially) compile-time validations as provider capabilities evolve.

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
- [x] **Tool Calling & Structured Outputs:**
  - Implement support for tool calls with JSON Schema validation. This includes integration of structured output support into the request builder with optional validation and fallback modes.
- [x] **Provider Preferences & Model Routing:**
  - Enable configuration for provider preferences, fallbacks, and routing options via the new strongly‑typed ProviderPreferences module.

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

With the new structured outputs support and provider preferences module, clients can now request responses that adhere to a specified JSON Schema and control routing behavior using strongly‑typed provider settings. Detailed error handling and the option to validate or fallback on unstructured output further enhance the robustness of the SDK.

This document provides an overview of the directory structure, design decisions, and our phased implementation plan. As the project evolves, additional endpoints and features will be introduced following this modular and test‑driven design.

---
