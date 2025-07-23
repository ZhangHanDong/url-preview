//! Browser-based content fetcher using MCP integration
//!
//! This module provides browser-based content fetching capabilities
//! for JavaScript-heavy websites and SPAs.

use crate::{PreviewError, Preview};
use crate::MetadataExtractor;
use crate::mcp_client::{McpClient, McpConfig, BrowserUsagePolicy};
use std::sync::Arc;
use url::Url;

#[cfg(feature = "logging")]
use tracing::{debug, warn, instrument};

/// Browser-based content fetcher
pub struct BrowserFetcher {
    /// MCP client for browser automation
    mcp_client: Arc<McpClient>,
    /// Browser usage policy
    usage_policy: BrowserUsagePolicy,
    /// Metadata extractor for preview generation
    metadata_extractor: Arc<MetadataExtractor>,
}

impl BrowserFetcher {
    /// Create a new browser fetcher
    pub fn new(config: McpConfig, usage_policy: BrowserUsagePolicy) -> Self {
        Self {
            mcp_client: Arc::new(McpClient::new(config)),
            usage_policy,
            metadata_extractor: Arc::new(MetadataExtractor::new()),
        }
    }
    
    /// Initialize the browser fetcher
    pub async fn initialize(&self) -> Result<(), PreviewError> {
        self.mcp_client.start().await
    }
    
    /// Shutdown the browser fetcher
    pub async fn shutdown(&self) -> Result<(), PreviewError> {
        self.mcp_client.stop().await
    }
    
    /// Check if browser should be used for this URL
    pub fn should_use_browser(&self, url: &str) -> bool {
        match self.usage_policy {
            BrowserUsagePolicy::Always => true,
            BrowserUsagePolicy::Never => false,
            BrowserUsagePolicy::Auto => self.detect_browser_need(url),
        }
    }
    
    /// Detect if browser is needed based on URL patterns
    fn detect_browser_need(&self, url: &str) -> bool {
        // Heuristics for detecting when browser rendering is needed
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };
        
        let domain = parsed.host_str().unwrap_or("");
        
        // Known SPAs and JavaScript-heavy sites
        let spa_domains = [
            "twitter.com", "x.com",
            "instagram.com",
            "facebook.com",
            "linkedin.com",
            "reddit.com",
            "discord.com",
            "slack.com",
            "notion.so",
            "vercel.app",
            "netlify.app",
            "web.app", // Firebase hosting
        ];
        
        // Check if domain matches known SPAs
        for spa_domain in &spa_domains {
            if domain.ends_with(spa_domain) {
                return true;
            }
        }
        
        // Check for common SPA frameworks in path
        let path = parsed.path();
        let spa_indicators = ["#/", "#!/", "/app/", "/dashboard/"];
        
        for indicator in &spa_indicators {
            if path.contains(indicator) {
                return true;
            }
        }
        
        false
    }
    
    /// Fetch content using browser
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn fetch_with_browser(&self, url: &str) -> Result<String, PreviewError> {
        #[cfg(feature = "logging")]
        debug!("Fetching content with browser for URL: {}", url);
        
        // Navigate to the URL
        self.mcp_client.navigate(url).await?;
        
        // Wait for page to load (simple implementation)
        // In production, we'd use better wait strategies
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Get the page HTML
        let html = self.mcp_client.get_page_html().await?;
        
        #[cfg(feature = "logging")]
        debug!("Successfully fetched {} bytes of HTML", html.len());
        
        Ok(html)
    }
    
    /// Generate preview using browser
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        // Fetch content with browser
        let html = self.fetch_with_browser(url).await?;
        
        // Extract metadata using metadata extractor
        self.metadata_extractor.extract(&html, url)
    }
    
    /// Take a screenshot of the page
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn take_screenshot(&self, url: &str) -> Result<Vec<u8>, PreviewError> {
        #[cfg(feature = "logging")]
        debug!("Taking screenshot of URL: {}", url);
        
        // Navigate to the URL if not already there
        self.mcp_client.navigate(url).await?;
        
        // Wait for page to load
        self.mcp_client.wait_for_load().await?;
        
        // Take screenshot
        self.mcp_client.take_screenshot().await
    }
    
    /// Extract structured data using JavaScript
    #[cfg_attr(feature = "logging", instrument(skip(self, script)))]
    pub async fn extract_with_script<T>(&self, url: &str, script: &str) -> Result<T, PreviewError>
    where
        T: serde::de::DeserializeOwned,
    {
        // Navigate to the URL
        self.mcp_client.navigate(url).await?;
        
        // Wait for page to load
        self.mcp_client.wait_for_load().await?;
        
        // Execute the extraction script
        let result = self.mcp_client.evaluate(script).await?;
        
        // Parse the result
        serde_json::from_value(result)
            .map_err(|e| PreviewError::ParseError(format!("Failed to parse extraction result: {}", e)))
    }
}

/// Browser-enhanced preview service
pub struct BrowserPreviewService {
    /// Browser fetcher
    browser_fetcher: Arc<BrowserFetcher>,
    /// Fallback fetcher for non-browser URLs
    fallback_fetcher: Arc<crate::Fetcher>,
    /// Metadata extractor
    metadata_extractor: Arc<MetadataExtractor>,
}

impl BrowserPreviewService {
    /// Create a new browser preview service
    pub fn new(mcp_config: McpConfig, usage_policy: BrowserUsagePolicy) -> Self {
        Self {
            browser_fetcher: Arc::new(BrowserFetcher::new(mcp_config, usage_policy)),
            fallback_fetcher: Arc::new(crate::Fetcher::new()),
            metadata_extractor: Arc::new(MetadataExtractor::new()),
        }
    }
    
    /// Initialize the service
    pub async fn initialize(&self) -> Result<(), PreviewError> {
        self.browser_fetcher.initialize().await
    }
    
    /// Check if browser should be used for this URL
    pub fn should_use_browser(&self, url: &str) -> bool {
        self.browser_fetcher.should_use_browser(url)
    }
    
    /// Generate preview with automatic browser detection
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn generate_preview(&self, url: &str) -> Result<Preview, PreviewError> {
        if self.browser_fetcher.should_use_browser(url) {
            #[cfg(feature = "logging")]
            debug!("Using browser for URL: {}", url);
            
            match self.browser_fetcher.generate_preview(url).await {
                Ok(preview) => Ok(preview),
                Err(_e) => {
                    #[cfg(feature = "logging")]
                    warn!("Browser fetch failed, falling back to standard fetch: {}", _e);
                    
                    // Fallback to standard fetching
                    let fetch_result = self.fallback_fetcher.fetch(url).await?;
                    let html = match fetch_result {
                        crate::FetchResult::Html(h) => h,
                        _ => return Err(PreviewError::InvalidContentType("Expected HTML".to_string())),
                    };
                    self.metadata_extractor.extract(&html, url)
                }
            }
        } else {
            #[cfg(feature = "logging")]
            debug!("Using standard fetch for URL: {}", url);
            
            let fetch_result = self.fallback_fetcher.fetch(url).await?;
            let html = match fetch_result {
                crate::FetchResult::Html(h) => h,
                _ => return Err(PreviewError::InvalidContentType("Expected HTML".to_string())),
            };
            self.metadata_extractor.extract(&html, url)
        }
    }
}

impl Drop for BrowserPreviewService {
    fn drop(&mut self) {
        // Ensure browser is shut down when service is dropped
        let fetcher = self.browser_fetcher.clone();
        tokio::spawn(async move {
            let _ = fetcher.shutdown().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_browser_detection() {
        let config = McpConfig::default();
        let fetcher = BrowserFetcher::new(config, BrowserUsagePolicy::Auto);
        
        // Test SPA detection
        assert!(fetcher.detect_browser_need("https://twitter.com/home"));
        assert!(fetcher.detect_browser_need("https://app.netlify.app"));
        assert!(fetcher.detect_browser_need("https://example.com/#!/page"));
        assert!(fetcher.detect_browser_need("https://app.example.com/dashboard/"));
        
        // Test non-SPA URLs
        assert!(!fetcher.detect_browser_need("https://example.com"));
        assert!(!fetcher.detect_browser_need("https://blog.example.com/post"));
    }
    
    #[test]
    fn test_usage_policy() {
        let config = McpConfig::default();
        
        // Always policy
        let fetcher = BrowserFetcher::new(config.clone(), BrowserUsagePolicy::Always);
        assert!(fetcher.should_use_browser("https://example.com"));
        
        // Never policy
        let fetcher = BrowserFetcher::new(config.clone(), BrowserUsagePolicy::Never);
        assert!(!fetcher.should_use_browser("https://twitter.com"));
        
        // Auto policy
        let fetcher = BrowserFetcher::new(config, BrowserUsagePolicy::Auto);
        assert!(fetcher.should_use_browser("https://twitter.com"));
        assert!(!fetcher.should_use_browser("https://example.com"));
    }
}