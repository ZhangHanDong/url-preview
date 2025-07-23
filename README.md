<div>
  <h1 align="center">url-preview</h1>
  <h4 align="center">
    🦀 A high-performance Rust library for generating rich URL previews with specialized support for Twitter/X and GitHub.
  </h4>
</div>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/url-preview.svg)](https://crates.io/crates/url-preview)
[![Documentation](https://docs.rs/url-preview/badge.svg)](https://docs.rs/url-preview)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

# URL Preview

A high-performance Rust library for generating rich URL previews with specialized support for Twitter/X and GitHub. This library offers efficient caching, concurrent processing, comprehensive metadata extraction, detailed error reporting, robust security features, browser-based rendering, and LLM-powered data extraction.

## What's New in v0.6.0

- **🌐 Browser-based Preview Generation**: JavaScript rendering with playwright-mcp integration
  - Automatic SPA detection and browser usage
  - Full DOM access after JavaScript execution
  - Screenshot capture capabilities
  - Custom JavaScript extraction
- **🤖 LLM-powered Structured Data Extraction**: Extract structured data using Large Language Models
  - **Multiple Provider Support**: OpenAI, Anthropic, claude-code-api, Local/Ollama, and Mock providers
  - **Type-safe Schema-driven Extraction**: Use Rust structs to define extraction schemas
  - **Smart Content Preprocessing**: HTML cleaning, Markdown conversion, and text extraction
  - **Auto-detection & Configuration**: Intelligent provider selection and environment-based setup
  - **Response Format Compatibility**: Works with both function calling and plain text responses
  - **API Key Validation**: Built-in validation for OpenAI and Anthropic API keys

## What's New in v0.5.0

- **🛡️ Comprehensive Security Framework**: Complete protection against web scraping vulnerabilities
  - SSRF protection with private IP and localhost blocking
  - Configurable URL validation and domain filtering
  - Content size and download time limits
  - Content type filtering to prevent malicious downloads
- **🔒 Secure by Default**: All security features are enabled by default
- **🎛️ Flexible Configuration**: Fine-grained control over security policies

## What's New in v0.4.0

- **Enhanced Error Handling**: New specific error types for better error differentiation
- **Invalid URL Detection**: Properly detects and reports 404s and invalid resources
- **Improved Performance**: Refactored internals for better concurrent processing
- **Better Feature Management**: Fixed compilation issues with optional features

## Features

- **High Performance**: Optimized for speed with concurrent processing and batch operations
- **Smart Caching**: Built-in DashMap-based caching system for lightning-fast responses
- **Platform-Specific Handlers**:
  - Twitter/X: Specialized handling with oEmbed support
  - GitHub: Enhanced repository information extraction with API integration
- **Comprehensive Security**:
  - SSRF protection against private IPs and localhost
  - URL scheme validation (http/https by default)
  - Domain whitelist/blacklist support
  - Content size and download time limits
  - Content type filtering
  - Protection against malicious redirects
- **Flexible Configuration**:
  - Customizable HTTP clients with different configurations
  - Adjustable concurrent request limits
  - Configurable cache sizes and strategies
  - Fine-grained security policy controls
- **Rich Metadata Extraction**:
  - Title, description, and images
  - Open Graph and Twitter Card metadata
  - Favicons and site information
- **Advanced Error Handling**:
  - Specific error types for DNS, timeout, and HTTP errors
  - Security-specific errors for blocked requests
  - Detailed error messages for debugging
  - Proper 404 and invalid resource detection
- **Modern Rust Features**:
  - Async/await with Tokio
  - Thread-safe components
  - Zero-cost abstractions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
url-preview = "0.6.0"

# Optional features
url-preview = { version = "0.6.0", features = ["full"] }

# Or select specific features
url-preview = { version = "0.6.0", features = ["cache", "logging", "github", "twitter", "browser", "llm"] }
```

### Feature Flags

- `default`: Basic functionality with default reqwest features
- `cache`: Enable caching support with DashMap
- `logging`: Enable structured logging with tracing
- `github`: Enable GitHub-specific preview enhancements
- `twitter`: Enable Twitter/X oEmbed integration
- `browser`: Enable browser-based rendering with playwright-mcp (requires Node.js)
- `llm`: Enable LLM-based data extraction
- `full`: Enable all features

## Quick Start

Here's a simple example to get started:

```rust
use url_preview::{PreviewService, Preview, PreviewError};

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    // Create a preview service with default settings
    let preview_service = PreviewService::new();

    // Generate a preview
    let preview = preview_service
        .generate_preview("https://www.rust-lang.org")
        .await?;

    println!("Title: {:?}", preview.title);
    println!("Description: {:?}", preview.description);
    println!("Image: {:?}", preview.image_url);

    Ok(())
}
```

## Advanced Usage

### Error Handling

The library now provides detailed error types for better error handling:

```rust
use url_preview::{PreviewService, PreviewError};

#[tokio::main]
async fn main() {
    let service = PreviewService::new();
    
    match service.generate_preview("https://example.com/404").await {
        Ok(preview) => println!("Got preview: {:?}", preview.title),
        Err(PreviewError::NotFound(msg)) => {
            println!("Resource not found: {}", msg);
        }
        Err(PreviewError::DnsError(msg)) => {
            println!("DNS resolution failed: {}", msg);
        }
        Err(PreviewError::TimeoutError(msg)) => {
            println!("Request timed out: {}", msg);
        }
        Err(PreviewError::ServerError { status, message }) => {
            println!("Server error ({}): {}", status, message);
        }
        Err(PreviewError::ClientError { status, message }) => {
            println!("Client error ({}): {}", status, message);
        }
        Err(e) => println!("Other error: {}", e),
    }
}
```

### Batch Processing

Process multiple URLs efficiently:

```rust
use url_preview::PreviewService;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let service = PreviewService::new();
    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com/rust-lang/rust",
        "https://news.ycombinator.com",
    ];

    // Concurrent processing with proper error handling
    let results = join_all(
        urls.iter().map(|url| service.generate_preview(url))
    ).await;

    for (url, result) in urls.iter().zip(results.iter()) {
        match result {
            Ok(preview) => println!("{}: {:?}", url, preview.title),
            Err(e) => println!("{}: Error - {}", url, e),
        }
    }
}
```

### Custom Configuration

Configure the service with specific requirements:

```rust
use url_preview::{PreviewService, PreviewServiceConfig, Fetcher, FetcherConfig};
use std::time::Duration;

let config = PreviewServiceConfig::new(1000) // Cache capacity
    .with_max_concurrent_requests(20)
    .with_default_fetcher(
        Fetcher::new_with_config(FetcherConfig {
            timeout: Duration::from_secs(30),
            user_agent: "my-app/1.0".into(),
            ..Default::default()
        })
    );

let service = PreviewService::new_with_config(config);
```

### Security Configuration

The library includes comprehensive security features that are enabled by default:

```rust
use url_preview::{
    PreviewService, PreviewServiceConfig, Fetcher, FetcherConfig,
    UrlValidationConfig, ContentLimits, CacheStrategy
};
use std::time::Duration;

// Configure URL validation
let mut url_validation = UrlValidationConfig::default();
url_validation.block_private_ips = true;  // Block private IPs (default)
url_validation.block_localhost = true;    // Block localhost (default)

// Add domain whitelist (only these domains will be allowed)
url_validation.allowed_domains.insert("trusted-site.com".to_string());
url_validation.allowed_domains.insert("api.example.com".to_string());

// Or use blacklist (these domains will be blocked)
// url_validation.blocked_domains.insert("malicious.com".to_string());

// Configure content limits
let mut content_limits = ContentLimits::default();
content_limits.max_content_size = 5 * 1024 * 1024;  // 5MB
content_limits.max_download_time = 20;               // 20 seconds
content_limits.allowed_content_types.insert("text/html".to_string());
content_limits.allowed_content_types.insert("application/json".to_string());

// Create fetcher with security config
let fetcher_config = FetcherConfig {
    url_validation,
    content_limits,
    timeout: Duration::from_secs(15),
    user_agent: "my-secure-app/1.0".to_string(),
    ..Default::default()
};

let service = PreviewService::new_with_config(
    PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(Fetcher::with_config(fetcher_config))
);

// Usage with security errors
match service.generate_preview("http://localhost:8080").await {
    Err(PreviewError::LocalhostBlocked) => {
        println!("Localhost access blocked for security");
    }
    Err(PreviewError::PrivateIpBlocked(ip)) => {
        println!("Private IP {} blocked", ip);
    }
    Err(PreviewError::DomainBlocked(domain)) => {
        println!("Domain {} is blacklisted", domain);
    }
    _ => {}
}
```

### Specialized Platform Support

#### Twitter/X Integration

```rust
#[cfg(feature = "twitter")]
{
    let preview = service
        .generate_preview("https://x.com/username/status/123456789")
        .await?;

    // Twitter previews include embedded content
    println!("Tweet content: {:?}", preview.description);
}
```

#### GitHub Repository Information

```rust
#[cfg(feature = "github")]
{
    // Basic preview
    let preview = service
        .generate_preview("https://github.com/rust-lang/rust")
        .await?;

    // Detailed repository information
    if let Ok(details) = service
        .get_github_detailed_info("https://github.com/rust-lang/rust")
        .await
    {
        println!("Stars: {}", details.stars_count);
        println!("Forks: {}", details.forks_count);
        println!("Language: {}", details.language);
        println!("Open Issues: {}", details.open_issues);
    }
}
```

### Caching Strategies

Control caching behavior for different use cases:

```rust
use url_preview::{PreviewService, CacheStrategy};

// Service with caching enabled (default)
let cached_service = PreviewService::new();

// Service with caching disabled
let no_cache_service = PreviewService::no_cache();

// Manual cache operations if needed
#[cfg(feature = "cache")]
{
    let preview = cached_service.generate_preview(url).await?;
    
    // Check if URL is in cache
    let cached = cached_service.default_generator.cache.get(url).await;
}
```

### Logging Configuration

Configure comprehensive logging:

```rust
#[cfg(feature = "logging")]
{
    use url_preview::{setup_logging, LogConfig};
    use std::path::PathBuf;

    let log_config = LogConfig {
        log_dir: PathBuf::from("logs"),
        log_level: "info".into(),
        console_output: true,
        file_output: true,
    };

    setup_logging(log_config);
}
```

## Performance Optimization

### Concurrent Request Limiting

Control the number of concurrent requests to prevent resource exhaustion:

```rust
use url_preview::{PreviewServiceConfig, MAX_CONCURRENT_REQUESTS};

let config = PreviewServiceConfig::new(1000)
    .with_max_concurrent_requests(50); // Default is MAX_CONCURRENT_REQUESTS (500)

let service = PreviewService::new_with_config(config);
```

### Retry Strategy

The library automatically retries on server errors and timeouts:

```rust
// The fetcher will automatically retry up to 3 times for:
// - Server errors (5xx)
// - Timeouts
// - Connection errors
// But NOT for client errors (4xx) or DNS failures
```

### Browser-based Preview Generation

Generate previews for JavaScript-heavy sites and SPAs:

```rust
#[cfg(feature = "browser")]
{
    use url_preview::{BrowserUsagePolicy, McpConfig, PreviewServiceConfig};
    
    // Configure browser integration
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@modelcontextprotocol/server-playwright".to_string(),
        ],
        ..Default::default()
    };
    
    let config = PreviewServiceConfig::new(1000)
        .with_mcp_config(mcp_config)
        .with_browser_usage_policy(BrowserUsagePolicy::Auto);
    
    let service = PreviewService::new_with_config(config);
    
    // Browser will automatically be used for SPAs
    let preview = service.generate_preview("https://twitter.com/rustlang").await?;
}
```

### LLM-powered Structured Data Extraction

Extract structured data from web pages using Large Language Models:

```rust
#[cfg(feature = "llm")]
{
    use url_preview::{LLMExtractor, LLMConfig, Fetcher};
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct ArticleInfo {
        title: String,
        summary: String,
        author: Option<String>,
        published_date: Option<String>,
        main_topics: Vec<String>,
        reading_time_minutes: Option<u32>,
    }
    
    // Auto-detect the best available provider
    let provider = LLMConfig::auto_detect_provider().await?;
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    // Extract structured article data
    let result = extractor.extract::<ArticleInfo>(
        "https://blog.rust-lang.org/2024/01/01/some-article.html",
        &fetcher
    ).await?;
    
    println!("Title: {}", result.data.title);
    println!("Topics: {:?}", result.data.main_topics);
    if let Some(usage) = result.usage {
        println!("Tokens used: {}", usage.total_tokens);
    }
}
```

#### Supported LLM Providers

```rust
use url_preview::{LLMConfig, OpenAIProvider, AnthropicProvider, MockProvider};

// OpenAI (requires OPENAI_API_KEY)
let provider = LLMConfig::openai_from_env()?;

// Anthropic (requires ANTHROPIC_API_KEY) 
let provider = LLMConfig::anthropic_from_env()?;

// claude-code-api (no API key needed)
let provider = LLMConfig::claude_code_from_env()?;

// Local models via Ollama (requires LOCAL_LLM_ENDPOINT)
let provider = LLMConfig::local_from_env()?;

// Mock provider for testing
let provider = MockProvider::new();
```

#### Content Processing Options

```rust
use url_preview::{LLMExtractorConfig, ContentFormat};

let config = LLMExtractorConfig {
    format: ContentFormat::Text,  // HTML, Markdown, or Text
    clean_html: true,             // Remove scripts, styles, nav elements
    max_content_length: 10_000,   // Limit content size
    model_params: Default::default(),
};

let extractor = LLMExtractor::with_config(provider, config);
```

## Examples

Check the `examples/` directory for comprehensive examples:

**Basic Usage:**
- `url_preview.rs` - Basic usage and caching demonstration
- `security_validation.rs` - Security features demonstration
- `content_limits.rs` - Content restriction examples
- `secure_preview_cli.rs` - CLI with full security options

**Platform-specific:**
- `github_preview.rs` - GitHub-specific features
- `twitter_preview.rs` - Twitter/X integration

**Advanced Features:**
- `browser_preview.rs` - Browser-based rendering for SPAs
- `browser_llm_extraction.rs` - Combined browser + LLM extraction

**LLM-powered Extraction:**
- `comprehensive_llm_test.rs` - Complete LLM functionality demonstration
- `test_anthropic_direct.rs` - Direct Anthropic API usage
- `test_openai_extraction.rs` - OpenAI API integration
- `claude_api_working.rs` - claude-code-api integration
- `llm_extraction.rs` - Basic LLM data extraction

**Performance:**
- `batch_concurrent.rs` - Batch processing examples
- `preview_cli.rs` - Interactive CLI example

Run examples with:

```bash
cargo run --example url_preview
cargo run --example security_validation
cargo run --example content_limits
cargo run --example secure_preview_cli -- https://example.com
cargo run --example github_preview --features github
cargo run --example twitter_preview --features twitter
```

## Benchmarks

The library includes comprehensive benchmarks:

```bash
cargo bench
```

## Testing

Run the test suite:

```bash
cargo test
cargo test --all-features  # Test with all features enabled
```

## Documentation

For detailed documentation, see:

- [API Documentation](https://docs.rs/url-preview) - Complete API reference
- [Security Guide](doc/security.md) - Comprehensive security documentation
- [Codebase Analysis](doc/codebase-analysis.md) - Architecture and implementation details

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`
4. Run benchmarks: `cargo bench`
5. Format code: `cargo fmt`
6. Run clippy: `cargo clippy -- -D warnings`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [scraper](https://crates.io/crates/scraper) for HTML parsing
- Uses [reqwest](https://crates.io/crates/reqwest) for HTTP requests
- Uses [dashmap](https://crates.io/crates/dashmap) for concurrent caching
- Uses [tokio](https://crates.io/crates/tokio) for async runtime

## Testing Websites for Rich Results

Google provides the [Rich Results Analysis Tool](https://search.google.com/test/rich-results?utm_source=support.google.com/webmasters/&utm_medium=referral&utm_campaign=7445569) to help you validate your website's tags.

Use this tool to ensure your website follows the conventions for optimal preview generation.