// src/mcp/client.rs
use crate::error::Result;
use crate::types::chat::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Model Context Protocol client for managing context windows
/// and efficient token usage across conversations.
pub struct MCPClient {
    /// Maximum context window size for the model
    max_context_size: usize,
    /// Strategy to use for context management
    strategy: Box<dyn ContextStrategy>,
    /// Context processor for complex operations
    processor: Option<Arc<dyn ContextProcessor>>,
}

/// Strategy for managing context to fit within token limits
pub trait ContextStrategy: Send + Sync {
    /// Fit messages within context window
    fn fit_to_context(&self, messages: Vec<Message>, max_tokens: usize) -> Result<Vec<Message>>;
    
    /// Compress messages to reduce token usage
    fn compress(&self, messages: Vec<Message>) -> Result<Vec<Message>>;
    
    /// Estimate token count for a set of messages or message subsets
    /// Default implementation uses a simple heuristic of 4 chars per token
    fn estimate_token_count(&self, messages: &[Message]) -> usize {
        messages.iter().map(|msg| {
            // Roughly 4 chars per token as a basic estimation
            (msg.content.len() / 4) + 5 // Add overhead for role, etc.
        }).sum()
    }
}

/// Advanced context processor interface
pub trait ContextProcessor: Send + Sync {
    /// Compress context using advanced techniques
    fn compress(&self, messages: Vec<Message>) -> Result<Vec<Message>>;
    
    /// Summarize previous context
    fn summarize(&self, messages: Vec<Message>) -> Result<Message>;
    
    /// Extract key information from context
    fn extract_key_info(&self, messages: Vec<Message>) -> Result<Vec<String>>;
}

impl MCPClient {
    /// Create a new MCP client with specified context size and strategy
    pub fn new(max_context_size: usize, strategy: Box<dyn ContextStrategy>) -> Self {
        Self {
            max_context_size,
            strategy,
            processor: None,
        }
    }
    
    /// Set a context processor for advanced operations
    pub fn with_processor(mut self, processor: Arc<dyn ContextProcessor>) -> Self {
        self.processor = Some(processor);
        self
    }
    
    /// Process messages to fit within context window
    pub fn process_context(&self, messages: &[Message]) -> Result<Vec<Message>> {
        let estimated_tokens = self.strategy.estimate_token_count(messages);
        
        if estimated_tokens <= self.max_context_size {
            return Ok(messages.to_vec());
        }
        
        // Apply the selected strategy to fit the context
        self.strategy.fit_to_context(messages.to_vec(), self.max_context_size)
    }
    
    /// Compress context using available processor
    pub fn compress_context(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        if let Some(processor) = &self.processor {
            processor.compress(messages)
        } else {
            // Default simple compression if no processor is available
            self.strategy.compress(messages)
        }
    }
    
    /// Manage a multi-part conversation that exceeds context limits
    pub async fn manage_conversation<F, Fut>(&self, 
        messages: Vec<Message>, 
        processor: F
    ) -> Result<ConversationResult> 
    where
        F: FnOnce(Vec<Message>) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<Message>>>
    {
        // Store the original message length for comparison
        let original_len = messages.len();
        
        // Process the context to fit within limits
        let processed_messages = self.process_context(&messages)?;
        
        // Process the conversation with the provided function
        let response_messages = processor(processed_messages.clone()).await?;
        
        // Get the token count before moving response_messages
        let token_count = self.strategy.estimate_token_count(&response_messages);
        
        // Return the result with metadata
        Ok(ConversationResult {
            messages: response_messages,
            compressed: processed_messages.len() < original_len,
            token_count,
        })
    }
}

/// Result of a managed conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResult {
    /// The resulting messages after processing
    pub messages: Vec<Message>,
    /// Whether the context was compressed
    pub compressed: bool,
    /// Estimated token count of the result
    pub token_count: usize,
}

