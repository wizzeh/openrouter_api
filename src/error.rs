use reqwest::Response;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error (status {code}): {message}")]
    ApiError {
        code: u16,
        message: String,
        metadata: Option<Value>,
    },

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates an API error from a response.
    pub async fn from_response(response: Response) -> Result<Self> {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        Ok(Error::ApiError {
            code: status,
            message: text,
            metadata: None,
        })
    }
}
