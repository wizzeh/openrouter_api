#[allow(unused)]
use crate::{
    types::chat::{ChatCompletionRequest, Message},
    OpenRouterClient, Result,
};
#[allow(unused)]
use std::env;

#[tokio::test]
async fn test_basic_chat_completion() -> Result<()> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set in the environment");

    // Make sure the base URL ends with a trailing slash.
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_http_referer("https://github.com/socrates8300/openrouter_api")
        .with_site_title("OpenRouter Rust SDK Tests")
        .with_api_key(api_key);

    // Use a model we expect to be available. Adjust if necessary.
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Say hello!".to_string(),
            name: None,
        }],
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    let response = client.chat_completion(request).await?;

    println!("Model used: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }
    println!("Usage: {:?}", response.usage);

    assert!(!response.choices.is_empty());
    assert_eq!(response.choices[0].message.role, "assistant");

    Ok(())
}
