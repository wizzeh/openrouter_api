OpenRouter API library that matches the documentation and follows Rust best practices. Here's the recommended structure with descriptions:

```
openrouter_api/
├── src/
│   ├── lib.rs                 # Library root, public exports, and high-level documentation
│   ├── client.rs              # OpenRouter client implementation
│   ├── config.rs              # Client configuration and builder
│   ├── models/                # Data models and types
│   │   ├── mod.rs            # Models module exports
│   │   ├── chat.rs           # Chat completion types
│   │   ├── completion.rs      # Text completion types
│   │   ├── common.rs         # Shared types between different endpoints
│   │   ├── error.rs          # Error types and conversions
│   │   └── parameters.rs      # LLM parameter types
│   ├── api/                   # API endpoint implementations
│   │   ├── mod.rs            # API module exports
│   │   ├── chat.rs           # Chat completion endpoint
│   │   ├── completion.rs      # Text completion endpoint
│   │   ├── generation.rs      # Generation metadata endpoint
│   │   ├── models.rs         # Available models endpoint
│   │   └── credits.rs        # Credits information endpoint
│   ├── types/                 # Shared type definitions
│   │   ├── mod.rs            # Types module exports
│   │   ├── provider.rs       # Provider-related types
│   │   ├── routing.rs        # Routing preference types
│   │   └── transform.rs      # Message transform types
│   └── utils/                 # Utility functions
│       ├── mod.rs            # Utils module exports
│       ├── http.rs           # HTTP-related utilities
│       ├── auth.rs           # Authentication utilities
│       └── validation.rs     # Input validation utilities
├── tests/                     # Integration tests
│   ├── common/               # Shared test utilities
│   │   ├── mod.rs           # Test utils exports
│   │   └── mock_server.rs   # Mock server for testing
│   ├── chat_tests.rs         # Chat completion tests
│   ├── completion_tests.rs   # Text completion tests
│   └── integration_tests.rs  # Full integration tests
├── examples/                  # Usage examples
│   ├── chat.rs              # Chat completion example
│   ├── completion.rs        # Text completion example
│   └── streaming.rs         # Streaming example
├── Cargo.toml                # Package manifest
├── README.md                 # Library documentation
├── CHANGELOG.md             # Version history
└── LICENSE                  # License information
```

Key design considerations:

1. **Modular Organization**: Each major feature has its own module, making the code easier to maintain and test.

2. **Clear Separation of Concerns**:
   - `models/` contains all data structures
   - `api/` handles endpoint implementations
   - `types/` contains shared type definitions
   - `utils/` contains reusable utilities

3. **Testing Structure**:
   - Integration tests are separated from unit tests
   - Mock server utilities for consistent testing
   - Common test utilities are shared

4. **Documentation**:
   - Examples directory for usage demonstrations
   - Comprehensive README and CHANGELOG
   - In-code documentation follows rustdoc standards

5. **Error Handling**:
   - Centralized error types in `models/error.rs`
   - Consistent error handling across all modules

This structure supports all the features mentioned in the documentation, including:
- Chat and completion endpoints
- Model routing
- Provider preferences
- Streaming support
- Tool calls
- Structured outputs
- Message transforms
- Authentication
- Error handling

Each module will be properly documented with Rust doc comments and will include appropriate unit tests alongside the implementation.

the architecture for a type-state builder pattern implementation for the OpenRouter client. This design ensures compile-time guarantees for required configurations.

```rust
// src/client/mod.rs
use std::marker::PhantomData;

// State markers
pub struct Unconfigured;
pub struct NoAuth;
pub struct Ready;

/// Main OpenRouter client struct using type-state pattern
pub struct OpenRouterClient<State = Unconfigured> {
    config: ClientConfig,
    http_client: Option<reqwest::Client>,
    _state: PhantomData<State>,
}

/// Configuration for the OpenRouter client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    api_key: Option<String>,
    base_url: String,
    http_referer: Option<String>,
    site_title: Option<String>,
    timeout: Option<Duration>,
}

impl Default for OpenRouterClient<Unconfigured> {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenRouterClient<Unconfigured> {
    /// Create a new unconfigured client builder
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
            http_client: None,
            _state: PhantomData,
        }
    }

    /// Configure base URL - transitions to NoAuth state
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> OpenRouterClient<NoAuth> {
        self.config.base_url = base_url.into();
        self.transition_to_no_auth()
    }

    fn transition_to_no_auth(self) -> OpenRouterClient<NoAuth> {
        OpenRouterClient {
            config: self.config,
            http_client: self.http_client,
            _state: PhantomData,
        }
    }
}

impl OpenRouterClient<NoAuth> {
    /// Add authentication - transitions to Ready state
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> OpenRouterClient<Ready> {
        self.config.api_key = Some(api_key.into());
        self.transition_to_ready()
    }

    /// Optional configurations that don't change state
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.config.http_referer = Some(referer.into());
        self
    }

    pub fn with_site_title(mut self, title: impl Into<String>) -> Self {
        self.config.site_title = Some(title.into());
        self
    }

    fn transition_to_ready(self) -> OpenRouterClient<Ready> {
        let http_client = reqwest::Client::builder()
            .timeout(self.config.timeout.unwrap_or(Duration::from_secs(30)))
            .build()
            .expect("Failed to create HTTP client");

        OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
        }
    }
}

impl OpenRouterClient<Ready> {
    /// Chat completion API
    pub fn chat(&self) -> chat::ChatApi {
        chat::ChatApi::new(self.http_client.clone().unwrap(), &self.config)
    }

    /// Text completion API
    pub fn completion(&self) -> completion::CompletionApi {
        completion::CompletionApi::new(self.http_client.clone().unwrap(), &self.config)
    }

    /// Models API
    pub fn models(&self) -> models::ModelsApi {
        models::ModelsApi::new(self.http_client.clone().unwrap(), &self.config)
    }

    /// Credits API
    pub fn credits(&self) -> credits::CreditsApi {
        credits::CreditsApi::new(self.http_client.clone().unwrap(), &self.config)
    }
}

// Usage example:
//
// let client = OpenRouterClient::new()
//     .with_base_url("https://openrouter.ai/api/v1")
//     .with_timeout(Duration::from_secs(60))
//     .with_http_referer("https://my-app.com")
//     .with_site_title("My App")
//     .with_api_key("your-api-key");
//
// let chat_api = client.chat();
// let completion_api = client.completion();
```

Then each API module would be structured like this:

```rust
// src/api/chat.rs
pub struct ChatApi {
    client: reqwest::Client,
    config: ClientConfig,
}

impl ChatApi {
    pub(crate) fn new(client: reqwest::Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    pub async fn create(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, Error> {
        // Implementation
    }

    pub async fn create_streaming(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk, Error>>, Error> {
        // Implementation
    }
}
```

Key benefits of this architecture:

1. **Type Safety**: The type-state pattern ensures that the client can't be used until properly configured

2. **Builder Pattern**: Fluent interface for configuration with clear state transitions

3. **Modular APIs**: Each API endpoint group is encapsulated in its own type

4. **Resource Management**: The HTTP client is created once and reused

5. **Ergonomic Usage**: Simple and intuitive API for end users

6. **Separation of Concerns**: Each API module handles its own requests/responses

This architecture ensures:
- Compile-time configuration validation
- Clear dependencies between configuration steps
- Immutable client state after construction
- Easy extension for new API endpoints
- Consistent error handling across all operations
- Efficient resource usage

The client can be extended with additional features while maintaining the type-state guarantees and clean API design.
