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

A high-performance Rust library for generating rich URL previews with specialized support for Twitter/X and GitHub. This library offers efficient caching, concurrent processing, comprehensive metadata extraction, and detailed error reporting.

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
- **Flexible Configuration**:
  - Customizable HTTP clients with different configurations
  - Adjustable concurrent request limits
  - Configurable cache sizes and strategies
- **Rich Metadata Extraction**:
  - Title, description, and images
  - Open Graph and Twitter Card metadata
  - Favicons and site information
- **Advanced Error Handling**:
  - Specific error types for DNS, timeout, and HTTP errors
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
url-preview = "0.4.0"

# Optional features
url-preview = { version = "0.4.0", features = ["full"] }

# Or select specific features
url-preview = { version = "0.4.0", features = ["cache", "logging", "github", "twitter"] }
```

### Feature Flags

- `default`: Basic functionality with default reqwest features
- `cache`: Enable caching support with DashMap
- `logging`: Enable structured logging with tracing
- `github`: Enable GitHub-specific preview enhancements
- `twitter`: Enable Twitter/X oEmbed integration
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

## Examples

Check the `examples/` directory for more comprehensive examples:

- `url_preview.rs` - Basic usage and caching demonstration
- `github_preview.rs` - GitHub-specific features
- `twitter_preview.rs` - Twitter/X integration
- `batch_concurrent.rs` - Batch processing examples
- `test_invalid_urls.rs` - Error handling examples

Run examples with:

```bash
cargo run --example url_preview
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