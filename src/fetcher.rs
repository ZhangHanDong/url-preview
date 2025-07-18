use super::is_twitter_url;
#[cfg(feature = "github")]
use crate::github_types::{GitHubBasicPreview, GitHubDetailedInfo, GitHubRepository};
use crate::PreviewError;
use reqwest::{header::HeaderMap, Client};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::time::Duration;
#[cfg(feature = "logging")]
use tracing::{debug, error, instrument, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct OEmbedResponse {
    pub html: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_url: String,
    pub provider_name: String,
    pub provider_url: String,
}

#[derive(Clone)]
pub struct Fetcher {
    client: Client,
}

#[derive(Debug, Clone)]
pub enum FetchResult {
    Html(String),
    OEmbed(OEmbedResponse),
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    pub fn new() -> Self {
        let user_agent = "url_preview/0.1.0";
        let timeout = Duration::from_secs(10);
        #[cfg(feature = "logging")]
        debug!("Fetcher initialized with default configuration");

        Self::new_with_custom_config(timeout, user_agent)
    }

    pub fn new_with_custom_config(timeout: Duration, user_agent: &str) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(user_agent)
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|e| {
                #[cfg(feature = "logging")]
                error!(error = %e, "Failed to create HTTP client");
                panic!("Failed to initialize HTTP client: {}", e);
            });
        Fetcher { client }
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_batch(&self, urls: Vec<&str>) -> Result<Vec<FetchResult>, PreviewError> {
        let futures: Vec<_> = urls.into_iter().map(|url| self.fetch(url)).collect();
        let results = futures::future::join_all(futures).await;

        let mut responses = Vec::new();
        for result in results {
            match result {
                Ok(response) => responses.push(response),
                Err(e) => return Err(e),
            }
        }

        Ok(responses)
    }

    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self), err))]
    pub async fn fetch_with_backoff(&self, url: &str) -> Result<String, PreviewError> {
        let max_retries = 3;
        let mut delay = Duration::from_millis(1000);

        for attempt in 0..max_retries {
            #[cfg(feature = "logging")]
            debug!(attempt = attempt + 1, "Attempting to fetch URL");

            match self.client.get(url).send().await {
                Ok(response) => {
                    // Check for 404 first
                    if response.status() == 404 {
                        return Err(PreviewError::NotFound(format!("Resource not found: {url}")));
                    }

                    if response.status().is_success() {
                        #[cfg(feature = "logging")]
                        debug!(url = %url, "Successfully fetched URL");
                        return response.text().await.map_err(|e| {
                            #[cfg(feature = "logging")]
                            error!(error = %e, "Failed to read response body");
                            PreviewError::FetchError(e.to_string())
                        });
                    }

                    // For server errors (5xx), retry
                    if response.status().is_server_error() && attempt < max_retries - 1 {
                        #[cfg(feature = "logging")]
                        warn!(
                            status = %response.status(),
                            attempt = attempt + 1,
                            "Server error, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }

                    // For client errors (4xx except 404) or final attempt, return error
                    let status = response.status().as_u16();
                    let message = format!("Server returned status: {}", response.status());
                    return Err(match status {
                        400..=499 => PreviewError::ClientError { status, message },
                        500..=599 => PreviewError::ServerError { status, message },
                        _ => PreviewError::HttpError { status, message },
                    });
                }
                Err(e) => {
                    let preview_error = PreviewError::from_reqwest_error(e);

                    // Only retry on server errors or timeouts
                    let should_retry = matches!(
                        &preview_error,
                        PreviewError::ServerError { .. }
                            | PreviewError::TimeoutError(_)
                            | PreviewError::ConnectionError(_)
                    );

                    if should_retry && attempt < max_retries - 1 {
                        #[cfg(feature = "logging")]
                        warn!(
                            error = %preview_error,
                            attempt = attempt + 1,
                            "Request error, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    #[cfg(feature = "logging")]
                    error!(error = %preview_error, "Request failed");
                    return Err(preview_error);
                }
            }
        }

        #[cfg(feature = "logging")]
        error!("Failed to fetch URL after maximum retries");
        Err(PreviewError::FetchError("Max retries exceeded".to_string()))
    }

    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self), err))]
    pub async fn fetch(&self, url: &str) -> Result<FetchResult, PreviewError> {
        #[cfg(feature = "logging")]
        debug!(url = %url, "Starting fetch request");

        if is_twitter_url(url) {
            #[cfg(feature = "logging")]
            debug!(url = %url, "Detected Twitter URL, using oEmbed API");
            #[cfg(feature = "twitter")]
            {
                let oembed = self.fetch_twitter_oembed(url).await?;
                Ok(FetchResult::OEmbed(oembed))
            }
            #[cfg(not(feature = "twitter"))]
            {
                // Fall back to regular HTML fetching
                self.fetch_html(url).await.map(FetchResult::Html)
            }
        } else {
            #[cfg(feature = "logging")]
            debug!(url = %url, "Fetching regular webpage");
            self.fetch_html(url).await.map(FetchResult::Html)
        }
    }

    async fn fetch_html(&self, url: &str) -> Result<String, PreviewError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            #[cfg(feature = "logging")]
            error!(error = %e, url = %url, "Failed to send request");
            PreviewError::from_reqwest_error(e)
        })?;

        // Check for 404 or other error status codes
        if response.status() == 404 {
            return Err(PreviewError::NotFound(format!("Resource not found: {url}")));
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("Server returned status: {}", response.status());

            return Err(match status {
                400..=499 => PreviewError::ClientError { status, message },
                500..=599 => PreviewError::ServerError { status, message },
                _ => PreviewError::HttpError { status, message },
            });
        }

        let content = response.text().await.map_err(|e| {
            #[cfg(feature = "logging")]
            error!(error = %e, url = %url, "Failed to read response body");
            PreviewError::FetchError(e.to_string())
        })?;

        #[cfg(feature = "logging")]
        debug!(url = %url, content_length = content.len(), "Successfully fetched webpage");
        Ok(content)
    }

    #[cfg(feature = "twitter")]
    #[cfg_attr(feature = "logging", instrument(level = "debug", skip(self), err))]
    async fn fetch_twitter_oembed(&self, tweet_url: &str) -> Result<OEmbedResponse, PreviewError> {
        let oembed_url = format!(
            "https://publish.twitter.com/oembed?url={}&omit_script=1&lang=en",
            tweet_url
        );

        #[cfg(feature = "logging")]
        debug!(tweet_url = %tweet_url, "Fetching Twitter oEmbed data");

        let response = self.client.get(&oembed_url).send().await.map_err(|e| {
            #[cfg(feature = "logging")]
            error!(error = %e, url = %tweet_url, "Failed to fetch Twitter oEmbed");
            // For external services, we wrap the specific error
            let inner_error = PreviewError::from_reqwest_error(e);
            match inner_error {
                PreviewError::DnsError(msg) => PreviewError::ExternalServiceError {
                    service: "Twitter".to_string(),
                    message: format!("DNS error: {}", msg),
                },
                PreviewError::TimeoutError(msg) => PreviewError::ExternalServiceError {
                    service: "Twitter".to_string(),
                    message: format!("Timeout: {}", msg),
                },
                PreviewError::ConnectionError(msg) => PreviewError::ExternalServiceError {
                    service: "Twitter".to_string(),
                    message: format!("Connection error: {}", msg),
                },
                _ => PreviewError::ExternalServiceError {
                    service: "Twitter".to_string(),
                    message: inner_error.to_string(),
                },
            }
        })?;

        // Check for 404 or other error status codes
        if response.status() == 404 {
            return Err(PreviewError::NotFound(format!(
                "Twitter/X content not found: {tweet_url}"
            )));
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("Twitter API returned status: {}", response.status());

            // For Twitter, we still wrap it as an external service error but include status info
            return Err(PreviewError::ExternalServiceError {
                service: "Twitter".to_string(),
                message: match status {
                    400..=499 => format!("Client error ({}): {}", status, message),
                    500..=599 => format!("Server error ({}): {}", status, message),
                    _ => format!("HTTP error ({}): {}", status, message),
                },
            });
        }

        let oembed: OEmbedResponse = response.json().await.map_err(|e| {
            #[cfg(feature = "logging")]
            error!(error = %e, url = %tweet_url, "Failed to parse Twitter oEmbed response");
            PreviewError::ExternalServiceError {
                service: "Twitter".to_string(),
                message: e.to_string(),
            }
        })?;

        #[cfg(feature = "logging")]
        debug!(tweet_url = %tweet_url, "Successfully fetched Twitter oEmbed data");
        Ok(oembed)
    }
}

// for Twitter
#[cfg(feature = "twitter")]
impl Fetcher {
    #[cfg_attr(feature = "logging", instrument(level = "debug"))]
    pub fn new_twitter_client() -> Self {
        #[cfg(feature = "logging")]
        debug!("Creating Twitter-specific fetcher");

        let mut headers = HeaderMap::new();

        headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
        headers.insert(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
                .parse()
                .unwrap(),
        );

        headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());

        headers.insert("Cache-Control", "no-cache".parse().unwrap());
        headers.insert("Pragma", "no-cache".parse().unwrap());

        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                AppleWebKit/537.36 (KHTML, like Gecko) \
                Chrome/119.0.0.0 Safari/537.36",
            )
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .default_headers(headers)
            .build()
            .expect("Failed to create Twitter HTTP client");

        #[cfg(feature = "logging")]
        debug!("Twitter-specific fetcher created successfully");
        Self { client }
    }

    /// Creates a Fetcher with custom configuration
    /// This method allows users to provide their own configuration options
    pub fn new_with_config(config: FetcherConfig) -> Self {
        let mut client_builder = Client::builder()
            .user_agent(config.user_agent)
            .timeout(config.timeout);

        // Apply custom headers
        if let Some(headers) = config.headers {
            client_builder = client_builder.default_headers(headers);
        }

        // Apply redirect policy
        if let Some(redirect_policy) = config.redirect_policy {
            client_builder = client_builder.redirect(redirect_policy);
        }

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client with custom config");

        Self { client }
    }
}

// for GitHub
#[cfg(feature = "github")]
impl Fetcher {
    pub fn new_github_client() -> Self {
        #[cfg(feature = "logging")]
        debug!("Creating GitHub-specific client");

        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            #[cfg(feature = "logging")]
            debug!("Found GitHub token in environment");
            headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        let client = Client::builder()
            .user_agent("url_preview/1.0")
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create GitHub HTTP client");

        Self { client }
    }

    pub async fn fetch_github_repo(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepository, PreviewError> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        #[cfg(feature = "logging")]
        debug!(url = %url, "Fetching GitHub repository information");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(PreviewError::from_reqwest_error)?;

        // Check for 404 or other error status codes
        if response.status() == 404 {
            return Err(PreviewError::NotFound(format!(
                "GitHub repository {owner}/{repo} not found"
            )));
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("API returned status: {}", response.status());

            return Err(match status {
                400..=499 => PreviewError::ClientError { status, message },
                500..=599 => PreviewError::ServerError { status, message },
                _ => PreviewError::HttpError { status, message },
            });
        }

        let repo_info: GitHubRepository = response
            .json()
            .await
            .map_err(|e| PreviewError::ParseError(e.to_string()))?;

        Ok(repo_info)
    }

    /// A helper function to extract GitHub owner and repo from URL
    /// Examples:
    /// - https://github.com/rust-lang/rust -> (rust-lang, rust)
    /// - https://github.com/rust-lang/rust/issues/123 -> (rust-lang, rust)
    pub fn parse_github_url(url: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = url
            .trim_start_matches("https://")
            .trim_start_matches("github.com/")
            .split('/')
            .collect();

        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }

        None
    }

    /// Extracts Open Graph image from HTML
    fn extract_og_image(html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("meta[property='og:image']").ok()?;

        document
            .select(&selector)
            .next()
            .and_then(|elem| elem.value().attr("content"))
            .map(|s| s.to_string())
    }

    /// Gets a basic preview using HTML scraping (no API key required)
    pub async fn fetch_github_basic_preview(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubBasicPreview, PreviewError> {
        let url = format!("https://github.com/{}/{}", owner, repo);
        #[cfg(feature = "logging")]
        debug!("Fetching basic preview for repository: {}/{}", owner, repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(PreviewError::from_reqwest_error)?;

        // Check for 404 or other error status codes
        if response.status() == 404 {
            return Err(PreviewError::NotFound(format!(
                "GitHub repository {owner}/{repo} not found"
            )));
        }

        if !response.status().is_success() {
            return Err(PreviewError::FetchError(format!(
                "GitHub returned status: {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| PreviewError::FetchError(e.to_string()))?;

        let document = Html::parse_document(&html);

        // Extract title, description, and image
        let title = Self::extract_meta_content(&document, "meta[property='og:title']");
        let description = Self::extract_meta_content(&document, "meta[property='og:description']");
        let image_url = Self::extract_og_image(&html);

        #[cfg(feature = "logging")]
        {
            if let Some(ref url) = image_url {
                debug!("Found GitHub Reop Preview Image URL: {}", url);
            } else {
                warn!("Not Found GitHub Reop Preview Image URL");
            }
        }

        Ok(GitHubBasicPreview {
            title,
            description,
            image_url,
        })
    }

    /// Gets detailed info using the GitHub API
    pub async fn fetch_github_detailed_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubDetailedInfo, PreviewError> {
        let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        #[cfg(feature = "logging")]
        debug!("Fetching detailed info from GitHub API: {}", api_url);

        let response = self
            .client
            .get(&api_url)
            .send()
            .await
            .map_err(PreviewError::from_reqwest_error)?;

        // Check for 404 or other error status codes
        if response.status() == 404 {
            return Err(PreviewError::NotFound(format!(
                "GitHub repository {owner}/{repo} not found"
            )));
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("API returned status: {}", response.status());

            return Err(match status {
                400..=499 => PreviewError::ClientError { status, message },
                500..=599 => PreviewError::ServerError { status, message },
                _ => PreviewError::HttpError { status, message },
            });
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PreviewError::ParseError(e.to_string()))?;

        Ok(GitHubDetailedInfo {
            full_name: data["full_name"].as_str().unwrap_or("").to_string(),
            description: data["description"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            stars_count: data["stargazers_count"].as_u64().unwrap_or(0) as u32,
            forks_count: data["forks_count"].as_u64().unwrap_or(0) as u32,
            open_issues_count: data["open_issues_count"].as_u64().unwrap_or(0) as u32,
            language: data["language"].as_str().map(|s| s.to_string()),
            default_branch: data["default_branch"]
                .as_str()
                .unwrap_or("main")
                .to_string(),
            topics: data["topics"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            html_url: data["html_url"].as_str().unwrap_or(&api_url).to_string(),
            homepage: data["homepage"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        })
    }

    fn extract_meta_content(document: &Html, selector_str: &str) -> Option<String> {
        let selector = Selector::parse(selector_str).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|elem| elem.value().attr("content"))
            .map(|s| s.to_string())
    }
}

// Helper functions that don't depend on features
impl Fetcher {
    pub fn extract_twitter_image_from_html(html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("meta[name='twitter:image']").ok()?;

        if let Some(url) = document
            .select(&selector)
            .next()
            .and_then(|elem| elem.value().attr("content"))
        {
            #[cfg(feature = "logging")]
            debug!("Found Twitter image URL: {}", url);
            return Some(url.to_string());
        }

        let og_selector = Selector::parse("meta[property='og:image']").ok()?;
        document
            .select(&og_selector)
            .next()
            .and_then(|elem| elem.value().attr("content"))
            .map(|url| {
                #[cfg(feature = "logging")]
                debug!("Found Open Graph image URL: {}", url);
                url.to_string()
            })
    }
}

/// Configuration for the Fetcher
pub struct FetcherConfig {
    pub user_agent: String,
    pub timeout: Duration,
    pub headers: Option<HeaderMap>,
    pub redirect_policy: Option<reqwest::redirect::Policy>,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            user_agent: "url_preview/0.1.0".to_string(),
            timeout: Duration::from_secs(10),
            headers: None,
            redirect_policy: None,
        }
    }
}
