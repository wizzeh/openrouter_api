pub mod api;
pub mod client;
pub mod error;
pub mod models;
pub mod tests;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

pub use client::{NoAuth, OpenRouterClient, Ready, Unconfigured};
