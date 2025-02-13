use async_trait::async_trait;

mod cache;
mod error;
mod extractor;
mod fetcher;
mod github_types;
mod logging;
mod preview_generator;
mod preview_service;
mod utils;

pub use cache::Cache;
pub use error::PreviewError;
pub use extractor::MetadataExtractor;
pub use fetcher::{FetchResult, Fetcher, FetcherConfig};
pub use logging::{log_error_card, log_preview_card, setup_logging, LogConfig, LogLevelGuard};
pub use preview_generator::UrlPreviewGenerator;
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

pub fn is_twitter_url(url: &str) -> bool {
    url.contains("twitter.com") || url.contains("x.com")
}
