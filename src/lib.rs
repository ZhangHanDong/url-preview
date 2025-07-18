//! # url-preview
//!
//! A high-performance Rust library for generating rich URL previews with specialized support 
//! for Twitter/X and GitHub.
//!
//! ## Features
//!
//! - **High Performance**: Optimized with concurrent processing and smart caching
//! - **Platform Support**: Specialized handlers for Twitter/X and GitHub
//! - **Rich Metadata**: Extract titles, descriptions, images, and more
//! - **Error Handling**: Detailed error types for better debugging
//! - **Flexible Configuration**: Customize timeouts, user agents, and caching
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use url_preview::{PreviewService, PreviewError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), PreviewError> {
//!     let service = PreviewService::new();
//!     let preview = service.generate_preview("https://www.rust-lang.org").await?;
//!     
//!     println!("Title: {:?}", preview.title);
//!     println!("Description: {:?}", preview.description);
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling (New in v0.4.0)
//!
//! The library now provides specific error types:
//!
//! ```rust,no_run
//! use url_preview::{PreviewService, PreviewError};
//!
//! # async fn example() {
//! let service = PreviewService::new();
//! match service.generate_preview("https://invalid.url").await {
//!     Ok(preview) => { /* handle preview */ },
//!     Err(PreviewError::NotFound(msg)) => println!("404: {}", msg),
//!     Err(PreviewError::DnsError(msg)) => println!("DNS failed: {}", msg),
//!     Err(PreviewError::TimeoutError(msg)) => println!("Timeout: {}", msg),
//!     Err(PreviewError::ServerError { status, message }) => {
//!         println!("Server error {}: {}", status, message)
//!     },
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # }
//! ```

use async_trait::async_trait;

#[cfg(feature = "cache")]
mod cache;
mod error;
mod extractor;
mod fetcher;
#[cfg(feature = "github")]
mod github_types;
#[cfg(feature = "logging")]
mod logging;
mod preview_generator;
mod preview_service;
mod utils;

#[cfg(feature = "cache")]
pub use cache::Cache;
pub use error::PreviewError;
pub use extractor::MetadataExtractor;
pub use fetcher::{FetchResult, Fetcher, FetcherConfig};
#[cfg(feature = "github")]
pub use github_types::{is_github_url, GitHubBasicPreview, GitHubDetailedInfo, GitHubRepository};
#[cfg(feature = "logging")]
pub use logging::{log_error_card, log_preview_card, setup_logging, LogConfig, LogLevelGuard};
pub use preview_generator::{CacheStrategy, UrlPreviewGenerator};
pub use preview_service::{PreviewService, PreviewServiceConfig, MAX_CONCURRENT_REQUESTS};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Preview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub favicon: Option<String>,
    pub site_name: Option<String>,
}

#[async_trait]
pub trait PreviewGenerator {
    async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError>;
}

#[cfg(feature = "twitter")]
pub fn is_twitter_url(url: &str) -> bool {
    url.contains("twitter.com") || url.contains("x.com")
}

#[cfg(not(feature = "twitter"))]
pub fn is_twitter_url(_url: &str) -> bool {
    false
}
