use super::is_twitter_url;
use crate::github_types::{GitHubBasicPreview, GitHubDetailedInfo, GitHubRepository};
use crate::PreviewError;
use reqwest::{header::HeaderMap, Client};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::time::Duration;
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

    #[instrument(level = "debug", skip(self), err)]
    pub async fn fetch_with_backoff(&self, url: &str) -> Result<String, PreviewError> {
        let max_retries = 3;
        let mut delay = Duration::from_millis(1000);

        for attempt in 0..max_retries {
            debug!(attempt = attempt + 1, "Attempting to fetch URL");

            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!(url = %url, "Successfully fetched URL");
                        return response.text().await.map_err(|e| {
                            error!(error = %e, "Failed to read response body");
                            PreviewError::FetchError(e.to_string())
                        });
                    }

                    if attempt < max_retries - 1 {
                        warn!(
                            status = %response.status(),
                            attempt = attempt + 1,
                            "Request failed, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                }
                Err(e) => {
                    if attempt < max_retries - 1 {
                        warn!(
                            error = %e,
                            attempt = attempt + 1,
                            "Request error, retrying after delay"
                        );
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    error!(error = %e, "Max retries exceeded");
                    return Err(PreviewError::FetchError(e.to_string()));
                }
            }
        }

        error!("Failed to fetch URL after maximum retries");
        Err(PreviewError::FetchError("Max retries exceeded".to_string()))
    }

    #[instrument(level = "debug", skip(self), err)]
    pub async fn fetch(&self, url: &str) -> Result<FetchResult, PreviewError> {
        debug!(url = %url, "Starting fetch request");

        if is_twitter_url(url) {
            debug!(url = %url, "Detected Twitter URL, using oEmbed API");
            let oembed = self.fetch_twitter_oembed(url).await?;
            Ok(FetchResult::OEmbed(oembed))
        } else {
            debug!(url = %url, "Fetching regular webpage");
            let content = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|e| {
                    error!(error = %e, url = %url, "Failed to send request");
                    PreviewError::FetchError(e.to_string())
                })?
                .text()
                .await
                .map_err(|e| {
                    error!(error = %e, url = %url, "Failed to read response body");
                    PreviewError::FetchError(e.to_string())
                })?;

            debug!(url = %url, content_length = content.len(), "Successfully fetched webpage");
            Ok(FetchResult::Html(content))
        }
    }

    #[instrument(level = "debug", skip(self), err)]
    async fn fetch_twitter_oembed(&self, tweet_url: &str) -> Result<OEmbedResponse, PreviewError> {
        let oembed_url = format!(
            "https://publish.twitter.com/oembed?url={}&omit_script=1&lang=en",
            tweet_url
        );

        debug!(tweet_url = %tweet_url, "Fetching Twitter oEmbed data");

        let response = self.client.get(&oembed_url).send().await.map_err(|e| {
            error!(error = %e, url = %tweet_url, "Failed to fetch Twitter oEmbed");
            PreviewError::ExternalServiceError {
                service: "Twitter".to_string(),
                message: e.to_string(),
            }
        })?;

        let oembed: OEmbedResponse = response.json().await.map_err(|e| {
            error!(error = %e, url = %tweet_url, "Failed to parse Twitter oEmbed response");
            PreviewError::ExternalServiceError {
                service: "Twitter".to_string(),
                message: e.to_string(),
            }
        })?;

        debug!(tweet_url = %tweet_url, "Successfully fetched Twitter oEmbed data");
        Ok(oembed)
    }
}

// for Twitter
impl Fetcher {
    #[instrument(level = "debug")]
    pub fn new_twitter_client() -> Self {
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
impl Fetcher {
    pub fn new_github_client() -> Self {
        debug!("Creating GitHub-specific client");

        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
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
            .unwrap();

        Self { client }
    }

    pub async fn fetch_github_repo(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepository, PreviewError> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        debug!(url = %url, "Fetching GitHub repository information");

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                PreviewError::FetchError(format!("GitHub API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(PreviewError::FetchError(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        response.json::<GitHubRepository>().await.map_err(|e| {
            PreviewError::ExtractError(format!("Failed to parse GitHub response: {}", e))
        })
    }
}

/// Creates a fetcher with Twitter-specific configurations.
///
/// # Examples
/// ```ignore
/// let fetcher = Fetcher::new();
///
/// // Using Twitter-specific configuration
/// let twitter_fetcher = Fetcher::new_twitter_client();
///
/// // Using custom configuration
/// let custom_fetcher = Fetcher::new_with_config(FetcherConfig {
///     user_agent: "my-custom-agent/1.0".to_string(),
///     timeout: Duration::from_secs(20),
///     headers: Some(my_custom_headers),
///     redirect_policy: Some(my_redirect_policy),
/// });
/// ```
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

// for GitHub
impl Fetcher {
    pub async fn fetch_github_basic_preview(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubBasicPreview, PreviewError> {
        let url = format!("https://github.com/{}/{}", owner, repo);
        debug!("Fetching basic preview for repository: {}/{}", owner, repo);

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                PreviewError::FetchError(format!("Failed to fetch GitHub page: {}", e))
            })?;

        let html = response.text().await.map_err(|e| {
            PreviewError::FetchError(format!("Failed to read response body: {}", e))
        })?;

        let document = Html::parse_document(&html);

        let title = self.extract_title(&document)?;
        let description = self.extract_description(&document);
        let image_url = self.extract_og_image(&document);

        if let Some(ref url) = image_url {
            debug!("Found GitHub Reop Preview Image URL: {}", url);
        } else {
            warn!("Not Found GitHub Reop Preview Image URL");
        }

        Ok(GitHubBasicPreview {
            title,
            description,
            image_url,
        })
    }

    pub async fn fetch_github_detailed_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubDetailedInfo, PreviewError> {
        let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        debug!("Fetching detailed info from GitHub API: {}", api_url);

        let response = self
            .client
            .get(&api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| PreviewError::FetchError(format!("GitHub API request failed: {}", e)))?;

        let repo_data: serde_json::Value = response.json().await.map_err(|e| {
            PreviewError::ExtractError(format!("Failed to parse GitHub API response: {}", e))
        })?;

        let contributors_url = format!("{}/contributors?per_page=1", api_url);
        let contributors_count = self.get_contributors_count(&contributors_url).await?;

        Ok(GitHubDetailedInfo {
            stars_count: repo_data["stargazers_count"].as_u64().unwrap_or(0) as u32,
            forks_count: repo_data["forks_count"].as_u64().unwrap_or(0) as u32,
            contributors_count,
            issues_count: repo_data["open_issues_count"].as_u64().unwrap_or(0) as u32,
            discussions_count: repo_data["discussions_count"].as_u64().unwrap_or(0) as u32,
            primary_language: repo_data["language"].as_str().map(String::from),
        })
    }

    fn extract_title(&self, document: &Html) -> Result<String, PreviewError> {
        let og_title_selector = Selector::parse("meta[property='og:title']")
            .map_err(|e| PreviewError::ExtractError(format!("Invalid selector: {}", e)))?;

        document
            .select(&og_title_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(String::from)
            .ok_or_else(|| PreviewError::ExtractError("Title not found".into()))
    }

    fn extract_description(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[property='og:description']").ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(String::from)
    }

    fn extract_og_image(&self, document: &Html) -> Option<String> {
        let twitter_image_selector = Selector::parse("meta[name='twitter:image']").ok()?;

        if let Some(url) = document
            .select(&twitter_image_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
        {
            debug!("Found Twitter image URL: {}", url);
            return Some(url.to_string());
        }

        // if not found twitter:image，back to find og:image
        let og_image_selector = Selector::parse("meta[property='og:image']").ok()?;

        document
            .select(&og_image_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|url| {
                debug!("Found Open Graph image URL: {}", url);
                url.to_string()
            })
    }

    async fn get_contributors_count(&self, url: &str) -> Result<u32, PreviewError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            PreviewError::FetchError(format!("Failed to fetch contributors: {}", e))
        })?;

        if let Some(link_header) = response.headers().get("Link") {
            if let Ok(link_str) = link_header.to_str() {
                if let Some(last_page) = parse_github_link_header(link_str) {
                    return Ok(last_page);
                }
            }
        }

        Ok(1)
    }
}

fn parse_github_link_header(link_str: &str) -> Option<u32> {
    // Link Header info：
    // <https://api.github.com/repos/owner/repo/contributors?page=2>; rel="next",
    // <https://api.github.com/repos/owner/repo/contributors?page=817>; rel="last"

    for link in link_str.split(',') {
        if link.contains("rel=\"last\"") {
            if let Some(page) = link
                .split(';')
                .next()
                .map(|url| url.trim_matches(|c| c == '<' || c == '>' || c == ' '))
                .and_then(|url| url.split('=').last())
                .and_then(|page| page.parse().ok())
            {
                return Some(page);
            }
        }
    }
    None
}
