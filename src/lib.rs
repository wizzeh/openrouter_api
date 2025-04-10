//! # OpenRouter API Client Library
//!
//! A production-ready Rust client for the OpenRouter API that provides access to multiple
//! leading AI models through a unified interface.
//!
//! ## Features
//!
//! - Type-safe API client with builder pattern
//! - Support for chat completions, text completions, and web search
//! - Model Coverage Profiles (MCP) for model selection and fallbacks
//! - Structured output support with JSON schema validation
//! - Tool calling capabilities for function execution
//! - Provider preferences for routing control
//! - Comprehensive error handling
//! - Streaming support for real-time responses
//!
//! ## Example
//!
//! ```rust
//! use openrouter_api::{OpenRouterClient, utils};
//! use openrouter_api::types::chat::Message;
//!
//! async fn example() -> Result<(), openrouter_api::Error> {
//!     // Load API key from environment
//!     let api_key = utils::load_api_key_from_env()?;
//!
//!     // Initialize client
//!     let client = OpenRouterClient::new()
//!         .with_base_url("https://openrouter.ai/api/v1/")?
//!         .with_http_referer("https://github.com/your-org/your-repo")
//!         .with_api_key(api_key)?;
//!
//!     // Create a simple chat request
//!     let chat_api = client.chat()?;
//!     let response = chat_api.simple_completion(
//!         "openai/gpt-4o",
//!         "Explain quantum computing in simple terms."
//!     ).await?;
//!
//!     println!("Response: {}", response);
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod error;
pub mod models;
pub mod tests;
pub mod types;
pub mod utils;
pub mod mcp;

pub use error::{Error, Result};
pub use types::*;

pub use client::{NoAuth, OpenRouterClient, Ready, Unconfigured};

// Re-export common types for convenience
pub use types::chat::{ChatCompletionRequest, ChatCompletionResponse, Message};
pub use types::completion::{CompletionRequest, CompletionResponse};
pub use types::routing::{ModelCoverageProfile, PredefinedModelCoverageProfile, ModelGroups};
pub use models::structured::JsonSchemaConfig;
pub use models::tool::{Tool, FunctionDescription};
pub use mcp::client::{MCPClient, ConversationResult};
pub use mcp::context::MCPClientFactory;
