use crate::github_types::{is_github_url, GitHubDetailedInfo};
use crate::{
    is_twitter_url, Fetcher, Preview, PreviewError, PreviewGenerator, UrlPreviewGenerator,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, instrument, warn};
use url::Url;

/// PreviewService provides a unified preview generation service
/// It can automatically identify different types of URLs and use appropriate processing strategies
#[derive(Clone)]
pub struct PreviewService {
    pub default_generator: Arc<UrlPreviewGenerator>,
    pub twitter_generator: Arc<UrlPreviewGenerator>,
    pub github_generator: Arc<UrlPreviewGenerator>,
    // Max Concurrent Requests
    semaphore: Arc<Semaphore>,
}

pub const MAX_CONCURRENT_REQUESTS: usize = 500;

impl Default for PreviewService {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewService {
    /// Creates a new preview service instance with default cache capacity
    pub fn new() -> Self {
        // Set 1000 cache entries for each generator
        // This means that up to 1000 different URL previews can be cached for each type (Normal/Twitter/GitHub)
        // 1000 cache entries take about 1-2MB memory
        // Total of 3-6MB for three generators is reasonable for modern systems
        Self::with_cache_cap(1000)
    }

    pub fn with_cache_cap(cache_capacity: usize) -> Self {
        debug!(
            "Initializing PreviewService with cache capacity: {}",
            cache_capacity
        );

        let default_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            Fetcher::new(),
        ));

        let twitter_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            Fetcher::new_twitter_client(),
        ));

        let github_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            Fetcher::new_github_client(),
        ));

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

        debug!("PreviewService initialized successfully");

        Self {
            default_generator,
            twitter_generator,
            github_generator,
            semaphore,
        }
    }

    pub fn new_with_config(config: PreviewServiceConfig) -> Self {
        debug!("Initializing PreviewService with custom configuration");

        let default_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config.default_fetcher.unwrap_or_default(),
        ));

        let twitter_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config
                .twitter_fetcher
                .unwrap_or_else(Fetcher::new_twitter_client),
        ));

        let github_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config
                .github_fetcher
                .unwrap_or_else(Fetcher::new_github_client),
        ));

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        debug!("PreviewService initialized with custom configuration");

        Self {
            default_generator,
            twitter_generator,
            github_generator,
            semaphore,
        }
    }

    fn extract_github_info(url: &str) -> Option<(String, String)> {
        let url = Url::parse(url).ok()?;
        if !url.host_str()?.contains("github.com") {
            return None;
        }

        let path_segments: Vec<&str> = url.path_segments()?.collect();
        if path_segments.len() >= 2 {
            Some((path_segments[0].to_string(), path_segments[1].to_string()))
        } else {
            None
        }
    }

    #[instrument(level = "debug", skip(self))]
    async fn generate_github_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        if let Some(cached) = self.github_generator.cache.get(url).await {
            return Ok(cached);
        }

        let (owner, repo_name) = Self::extract_github_info(url).ok_or_else(|| {
            warn!("GitHub URL parsing failed: {}", url);
            PreviewError::ExtractError("Invalid GitHub URL format".into())
        })?;

        match self
            .github_generator
            .fetcher
            .fetch_github_basic_preview(&owner, &repo_name)
            .await
        {
            Ok(basic_info) => {
                debug!("Found GitHub Repo {}/{} basic infos", owner, repo_name);

                let preview = Preview {
                    url: url.to_string(),
                    title: Some(basic_info.title),
                    description: basic_info.description,
                    image_url: basic_info.image_url,
                    site_name: Some("GitHub".to_string()),
                    favicon: Some(
                        "https://github.githubassets.com/favicons/favicon.svg".to_string(),
                    ),
                };

                self.github_generator
                    .cache
                    .set(url.to_string(), preview.clone())
                    .await;

                Ok(preview)
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Failed to get GitHub basic preview, will use general preview generator as fallback"
                );
                self.github_generator.generate_preview(url).await
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        debug!("Starting preview generation for URL: {}", url);



        // match &result {
        //     Ok(preview) => {
        //         log_preview_card(preview, url);
        //     }
        //     Err(e) => {
        //         log_error_card(url, e);
        //     }
        // }

        if is_twitter_url(url) {
            debug!("Detected Twitter URL, using specialized handler");
            self.twitter_generator.generate_preview(url).await
        } else if is_github_url(url) {
            debug!("Detected GitHub URL, using specialized handler");
            self.generate_github_preview(url).await
        } else {
            debug!("Using default URL handler");
            self.default_generator.generate_preview(url).await
        }
    }

    pub async fn generate_github_basic_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        let (owner, repo) = Self::extract_github_info(url)
            .ok_or_else(|| PreviewError::ExtractError("Invalid GitHub URL format".into()))?;

        let basic_info = self
            .github_generator
            .fetcher
            .fetch_github_basic_preview(&owner, &repo)
            .await?;

        Ok(Preview {
            url: url.to_string(),
            title: Some(basic_info.title),
            description: basic_info.description,
            image_url: basic_info.image_url,
            site_name: Some("GitHub".to_string()),
            favicon: Some("https://github.githubassets.com/favicons/favicon.svg".to_string()),
        })
    }

    pub async fn get_github_detailed_info(
        &self,
        url: &str,
    ) -> Result<GitHubDetailedInfo, PreviewError> {
        let (owner, repo) = Self::extract_github_info(url)
            .ok_or_else(|| PreviewError::ExtractError("Invalid GitHub URL format".into()))?;

        self.github_generator
            .fetcher
            .fetch_github_detailed_info(&owner, &repo)
            .await
    }
}

impl PreviewService {
    pub fn new_with_concurrency(config: PreviewServiceConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        let default_generator = Arc::new(UrlPreviewGenerator::new(config.cache_capacity));
        let twitter_generator = Arc::new(UrlPreviewGenerator::new(config.cache_capacity));
        let github_generator = Arc::new(UrlPreviewGenerator::new(config.cache_capacity));

        PreviewService {
            default_generator,
            twitter_generator,
            github_generator,
            semaphore,
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn generate_preview_with_concurrency(
        &self,
        url: &str,
    ) -> Result<Preview, PreviewError> {
        let permit = self.semaphore.clone().acquire_owned().await;
        let preview = self.generate_preview(url).await;
        drop(permit);
        preview
    }
}

#[derive(Default, Clone)]
pub struct PreviewServiceConfig {
    pub cache_capacity: usize,
    pub default_fetcher: Option<Fetcher>,
    pub twitter_fetcher: Option<Fetcher>,
    pub github_fetcher: Option<Fetcher>,
    pub max_concurrent_requests: usize,
}

impl PreviewServiceConfig {
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            cache_capacity,
            default_fetcher: None,
            twitter_fetcher: None,
            github_fetcher: None,
            max_concurrent_requests: MAX_CONCURRENT_REQUESTS,
        }
    }

    pub fn with_github_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.github_fetcher = Some(fetcher);
        self
    }

    pub fn with_default_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.default_fetcher = Some(fetcher);
        self
    }

    pub fn with_twitter_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.twitter_fetcher = Some(fetcher);
        self
    }
}
