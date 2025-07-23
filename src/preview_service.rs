#[cfg(feature = "github")]
use crate::github_types::{is_github_url, GitHubDetailedInfo};
use crate::{
    is_twitter_url, CacheStrategy, Fetcher, Preview, PreviewError, PreviewGenerator,
    UrlPreviewGenerator,
};
#[cfg(feature = "browser")]
use crate::browser_fetcher::BrowserPreviewService;
#[cfg(feature = "browser")]
use crate::mcp_client::{McpConfig, BrowserUsagePolicy};
use std::sync::Arc;
use tokio::sync::Semaphore;
#[cfg(all(feature = "logging", feature = "github"))]
use tracing::warn;
#[cfg(feature = "logging")]
use tracing::{debug, instrument};
use url::Url;

/// PreviewService provides a unified preview generation service
/// It can automatically identify different types of URLs and use appropriate processing strategies
#[derive(Clone)]
pub struct PreviewService {
    pub default_generator: Arc<UrlPreviewGenerator>,
    #[cfg(feature = "twitter")]
    pub twitter_generator: Arc<UrlPreviewGenerator>,
    #[cfg(feature = "github")]
    pub github_generator: Arc<UrlPreviewGenerator>,
    #[cfg(feature = "browser")]
    pub browser_service: Option<Arc<BrowserPreviewService>>,
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
        #[cfg(feature = "logging")]
        debug!(
            "Initializing PreviewService with cache capacity: {}",
            cache_capacity
        );

        let default_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            CacheStrategy::UseCache,
            Fetcher::new(),
        ));

        #[cfg(feature = "twitter")]
        let twitter_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            CacheStrategy::UseCache,
            Fetcher::new_twitter_client(),
        ));

        #[cfg(feature = "github")]
        let github_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            cache_capacity,
            CacheStrategy::UseCache,
            Fetcher::new_github_client(),
        ));

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

        #[cfg(feature = "logging")]
        debug!("PreviewService initialized successfully");

        Self {
            default_generator,
            #[cfg(feature = "twitter")]
            twitter_generator,
            #[cfg(feature = "github")]
            github_generator,
            #[cfg(feature = "browser")]
            browser_service: None,
            semaphore,
        }
    }

    pub fn no_cache() -> Self {
        #[cfg(feature = "logging")]
        debug!("Initializing PreviewService with cache capacity: {}", 0);

        let default_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            0,
            CacheStrategy::NoCache,
            Fetcher::new(),
        ));

        #[cfg(feature = "twitter")]
        let twitter_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            0,
            CacheStrategy::NoCache,
            Fetcher::new_twitter_client(),
        ));

        #[cfg(feature = "github")]
        let github_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            0,
            CacheStrategy::NoCache,
            Fetcher::new_github_client(),
        ));

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

        #[cfg(feature = "logging")]
        debug!("PreviewService initialized successfully");

        Self {
            default_generator,
            #[cfg(feature = "twitter")]
            twitter_generator,
            #[cfg(feature = "github")]
            github_generator,
            #[cfg(feature = "browser")]
            browser_service: None,
            semaphore,
        }
    }

    pub fn new_with_config(config: PreviewServiceConfig) -> Self {
        #[cfg(feature = "logging")]
        debug!("Initializing PreviewService with custom configuration");

        let default_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config.cache_strategy,
            config.default_fetcher.unwrap_or_default(),
        ));

        #[cfg(feature = "twitter")]
        let twitter_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config.cache_strategy,
            config
                .twitter_fetcher
                .unwrap_or_else(Fetcher::new_twitter_client),
        ));

        #[cfg(feature = "github")]
        let github_generator = Arc::new(UrlPreviewGenerator::new_with_fetcher(
            config.cache_capacity,
            config.cache_strategy,
            config
                .github_fetcher
                .unwrap_or_else(Fetcher::new_github_client),
        ));

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        #[cfg(feature = "browser")]
        let browser_service = if let Some(mcp_config) = config.mcp_config {
            Some(Arc::new(BrowserPreviewService::new(
                mcp_config,
                config.browser_usage_policy,
            )))
        } else {
            None
        };

        #[cfg(feature = "logging")]
        debug!("PreviewService initialized with custom configuration");

        Self {
            default_generator,
            #[cfg(feature = "twitter")]
            twitter_generator,
            #[cfg(feature = "github")]
            github_generator,
            #[cfg(feature = "browser")]
            browser_service,
            semaphore,
        }
    }

    #[cfg(feature = "github")]
    fn extract_github_info(url: &str) -> Option<(String, String)> {
        let parsed_url = Url::parse(url).ok()?;
        if !parsed_url.host_str()?.contains("github.com") {
            return None;
        }

        let path_segments: Vec<&str> = parsed_url.path_segments()?.collect();
        if path_segments.len() >= 2 {
            return Some((path_segments[0].to_string(), path_segments[1].to_string()));
        }
        None
    }

    #[cfg(feature = "github")]
    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self)))]
    async fn generate_github_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        #[cfg(feature = "cache")]
        if let CacheStrategy::UseCache = self.github_generator.cache_strategy {
            if let Some(cached) = self.github_generator.cache.get(url).await {
                return Ok(cached);
            }
        }

        let (owner, repo_name) = Self::extract_github_info(url).ok_or_else(|| {
            #[cfg(feature = "logging")]
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
                #[cfg(feature = "logging")]
                debug!("Found GitHub Repo {}/{} basic infos", owner, repo_name);

                let preview = Preview {
                    url: url.to_string(),
                    title: basic_info.title,
                    description: basic_info.description,
                    image_url: basic_info.image_url,
                    site_name: Some("GitHub".to_string()),
                    favicon: Some(
                        "https://github.githubassets.com/favicons/favicon.svg".to_string(),
                    ),
                };

                #[cfg(feature = "cache")]
                if let CacheStrategy::UseCache = self.github_generator.cache_strategy {
                    self.github_generator
                        .cache
                        .set(url.to_string(), preview.clone())
                        .await;
                }

                Ok(preview)
            }
            Err(_e) => {
                #[cfg(feature = "logging")]
                warn!(
                    error = ?_e,
                    "Failed to get GitHub basic preview, will use general preview generator as fallback"
                );
                self.github_generator.generate_preview(url).await
            }
        }
    }

    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self)))]
    pub async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        #[cfg(feature = "logging")]
        debug!("Starting preview generation for URL: {}", url);

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| PreviewError::ConcurrencyLimitError)?;

        let _ = Url::parse(url)
            .map_err(|e| PreviewError::ParseError(format!("Invalid URL format: {e}")))?;
        
        // Try browser service first if available
        #[cfg(feature = "browser")]
        if let Some(browser_service) = &self.browser_service {
            if browser_service.should_use_browser(url) {
                #[cfg(feature = "logging")]
                debug!("Using browser service for URL: {}", url);
                
                match browser_service.generate_preview(url).await {
                    Ok(preview) => return Ok(preview),
                    Err(_e) => {
                        #[cfg(feature = "logging")]
                        debug!("Browser service failed, falling back: {}", _e);
                    }
                }
            }
        }

        if is_twitter_url(url) {
            #[cfg(feature = "logging")]
            debug!("Detected Twitter URL, using specialized handler");
            #[cfg(feature = "twitter")]
            {
                self.twitter_generator.generate_preview(url).await
            }
            #[cfg(not(feature = "twitter"))]
            {
                self.default_generator.generate_preview(url).await
            }
        } else if cfg!(feature = "github") && {
            #[cfg(feature = "github")]
            {
                is_github_url(url)
            }
            #[cfg(not(feature = "github"))]
            {
                false
            }
        } {
            #[cfg(feature = "logging")]
            debug!("Detected GitHub URL, using specialized handler");
            #[cfg(feature = "github")]
            {
                self.generate_github_preview(url).await
            }
            #[cfg(not(feature = "github"))]
            {
                self.default_generator.generate_preview(url).await
            }
        } else {
            #[cfg(feature = "logging")]
            debug!("Using default URL handler");
            self.default_generator.generate_preview(url).await
        }
    }

    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self)))]
    pub async fn generate_preview_with_concurrency(
        &self,
        url: &str,
    ) -> Result<Preview, PreviewError> {
        #[cfg(feature = "logging")]
        debug!("Starting preview generation for URL: {}", url);

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| PreviewError::ConcurrencyLimitError)?;

        let _ = Url::parse(url)
            .map_err(|e| PreviewError::ParseError(format!("Invalid URL format: {e}")))?;

        if is_twitter_url(url) {
            #[cfg(feature = "logging")]
            debug!("Detected Twitter URL, using specialized handler");
            #[cfg(feature = "twitter")]
            {
                self.twitter_generator.generate_preview(url).await
            }
            #[cfg(not(feature = "twitter"))]
            {
                self.default_generator.generate_preview(url).await
            }
        } else if cfg!(feature = "github") && {
            #[cfg(feature = "github")]
            {
                is_github_url(url)
            }
            #[cfg(not(feature = "github"))]
            {
                false
            }
        } {
            #[cfg(feature = "logging")]
            debug!("Detected GitHub URL, using specialized handler");
            #[cfg(feature = "github")]
            {
                self.generate_github_preview(url).await
            }
            #[cfg(not(feature = "github"))]
            {
                self.default_generator.generate_preview(url).await
            }
        } else {
            #[cfg(feature = "logging")]
            debug!("Using default URL handler");
            self.default_generator.generate_preview(url).await
        }
    }

    #[cfg(feature = "github")]
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
            title: basic_info.title,
            description: basic_info.description,
            image_url: basic_info.image_url,
            site_name: Some("GitHub".to_string()),
            favicon: Some("https://github.githubassets.com/favicons/favicon.svg".to_string()),
        })
    }

    #[cfg(feature = "github")]
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

/// Static constructor methods
impl PreviewService {
    /// Create a new preview service for unit testing
    pub fn new_minimal() -> Self {
        let default_generator = Arc::new(UrlPreviewGenerator::new(100, CacheStrategy::UseCache));
        #[cfg(feature = "twitter")]
        let twitter_generator = Arc::new(UrlPreviewGenerator::new(100, CacheStrategy::UseCache));
        #[cfg(feature = "github")]
        let github_generator = Arc::new(UrlPreviewGenerator::new(100, CacheStrategy::UseCache));

        Self {
            default_generator,
            #[cfg(feature = "twitter")]
            twitter_generator,
            #[cfg(feature = "github")]
            github_generator,
            #[cfg(feature = "browser")]
            browser_service: None,
            semaphore: Arc::new(Semaphore::new(10)),
        }
    }

    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self)))]
    pub async fn generate_preview_no_cache(&self, url: &str) -> Result<Preview, PreviewError> {
        let generator = UrlPreviewGenerator::new_with_fetcher(
            0,
            CacheStrategy::NoCache,
            self.default_generator.fetcher.clone(),
        );
        generator.generate_preview(url).await
    }
}

pub struct PreviewServiceConfig {
    pub cache_capacity: usize,
    pub cache_strategy: CacheStrategy,
    pub max_concurrent_requests: usize,
    pub default_fetcher: Option<Fetcher>,
    #[cfg(feature = "twitter")]
    pub twitter_fetcher: Option<Fetcher>,
    #[cfg(feature = "github")]
    pub github_fetcher: Option<Fetcher>,
    #[cfg(feature = "browser")]
    pub mcp_config: Option<McpConfig>,
    #[cfg(feature = "browser")]
    pub browser_usage_policy: BrowserUsagePolicy,
}

impl PreviewServiceConfig {
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            cache_capacity,
            cache_strategy: CacheStrategy::UseCache,
            max_concurrent_requests: MAX_CONCURRENT_REQUESTS,
            default_fetcher: None,
            #[cfg(feature = "twitter")]
            twitter_fetcher: None,
            #[cfg(feature = "github")]
            github_fetcher: None,
            #[cfg(feature = "browser")]
            mcp_config: None,
            #[cfg(feature = "browser")]
            browser_usage_policy: BrowserUsagePolicy::Auto,
        }
    }

    #[cfg(feature = "github")]
    pub fn with_github_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.github_fetcher = Some(fetcher);
        self
    }

    pub fn with_default_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.default_fetcher = Some(fetcher);
        self
    }

    #[cfg(feature = "twitter")]
    pub fn with_twitter_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.twitter_fetcher = Some(fetcher);
        self
    }

    pub fn with_max_concurrent_requests(mut self, max_concurrent_requests: usize) -> Self {
        self.max_concurrent_requests = max_concurrent_requests;
        self
    }

    pub fn with_cache_strategy(mut self, cache_strategy: CacheStrategy) -> Self {
        self.cache_strategy = cache_strategy;
        self
    }
    
    #[cfg(feature = "browser")]
    pub fn with_mcp_config(mut self, mcp_config: McpConfig) -> Self {
        self.mcp_config = Some(mcp_config);
        self
    }
    
    #[cfg(feature = "browser")]
    pub fn with_browser_usage_policy(mut self, policy: BrowserUsagePolicy) -> Self {
        self.browser_usage_policy = policy;
        self
    }
}
