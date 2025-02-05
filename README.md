# OpenRouter API Client Library

OpenRouter API Client Library is a Rust client for interfacing with the OpenRouter API. The library is designed to be modular, type‑safe, and intuitive. It uses a type‑state builder pattern for configuring and validating the client at compile time, ensuring that all required configuration (such as setting the base URL and API key) happens before attempting a request.

> **Note:** This project is still in development. Many features are planned but not yet implemented.

## Features

- **Modular Organization:** Structure divided into models, API endpoints, types, and utility functions.
- **Type‑State Builder:** Guarantees compile‑time validation of client configuration.
- **HTTP Integration:** Uses [reqwest](https://crates.io/crates/reqwest) with rustls‑tls for secure asynchronous HTTP requests.
- **Robust Error Handling:** Centralized error module using `thiserror` for consistent error types.
- **Future Roadmap:**
  - Streaming support for real‑time completions.
  - Tool calling capability and structured outputs.
  - Model routing and provider preferences.
  - Additional endpoints such as credits, generation metadata, and available models.
  - Extended tests and documentation improvements.

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
openrouter_api = { git = "https://github.com/yourusername/openrouter_api.git", branch = "main" }
```

Ensure that you have Rust installed (tested with Rust v1.83.0) and that you're using Cargo for building and testing.

### Example Usage

Below is a minimal example that creates a client, configures it with the API key, and sends a simple chat completion request.

```rust
use openrouter_api::{
    OpenRouterClient, Ready,
    types::chat::{ChatCompletionRequest, Message},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Make sure to set your API key in an environment variable.
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set");

    // Create an OpenRouter client.
    let client = OpenRouterClient::new()
        // The base URL must have a trailing slash.
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_http_referer("https://yourwebsite.com")
        .with_site_title("Your Application")
        .with_api_key(api_key);

    // Create a chat completion request using an available model.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "What is the meaning of life?".to_string(),
                name: None,
            }
        ],
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    // Invoke the chat completion endpoint.
    let response = client.chat_completion(request).await?;

    println!("Model used: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }
    println!("Usage: {:?}", response.usage);

    Ok(())
}
```

### Running Tests

Before running tests, set the environment variable OPENROUTER_API_KEY with your API key:

```bash
export OPENROUTER_API_KEY=sk-...
cargo test
```

This will run the integration test in `tests/integration_tests.rs`. If the API key and model are correctly set, the test will complete and print the returned model, response message, and usage metadata.

## Implementation Plan

The project is still under active development. Below is an implementation roadmap outlining the upcoming features and milestones.

### Phase 1: Core Functionality (Completed or In Progress)
- [x] **Client Foundation:**
  - Implement the type‑state builder for configuration.
  - Validate required parameters at compile time.
  - Build a basic HTTP client with custom headers.
- [x] **Chat Completion Endpoint:**
  - Implement the `chat_completion` method using the correct URL joining logic.
  - Provide basic error handling (e.g., decoding response JSON).
- [x] **Core Data Models:**
  - Implement types for chat messages, requests, responses, and usage.

### Phase 2: Additional Endpoints and Features
- [ ] **Streaming Support:**
  - Implement streaming versions of the chat completions (e.g., `create_chat_completion_stream` using Server-Sent Events).
- [ ] **Text Completion Endpoint:**
  - Create an API module and associated types for text completions.
- [ ] **Models & Credits Endpoints:**
  - Implement endpoints to list available models and retrieve credits information.
- [ ] **Tool Calling & Structured Outputs:**
  - Add support for tool calls and structured responses, using JSON Schema validation.
- [ ] **Provider Preferences & Model Routing:**
  - Enable specification of provider preferences, fallbacks, and routing options.

### Phase 3: Robust Testing & Documentation
- [ ] **Unit & Integration Tests:**
  - Expand test coverage with mocks and integration tests for each endpoint.
- [ ] **Documentation Improvements:**
  - Enhance inline documentation and generate API docs using rustdoc.
  - Provide usage examples in the `examples/` directory.
- [ ] **Continuous Integration (CI):**
  - Set up CI pipeline for automated builds and tests.

## Contributing

Contributions are welcome! If you have ideas or improvements, please open an issue or submit a pull request. Please adhere to the code style and ensure tests pass.

## License

Distributed under either the MIT license or the Apache License, Version 2.0. See [LICENSE](LICENSE) for more information.

–––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––



## Verbose output for debugging
cargo test -- --nocapture
