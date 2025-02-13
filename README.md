<div>
  <h1 align="center">url-preview</h1>
  <h4 align="center">
    🦀 A high-performance Rust library for generating rich URL previews with specialized support for Twitter/X and GitHub.
  </h4>
</div>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/link-preview.svg)](https://crates.io/crates/url-preview)
[![Documentation](https://docs.rs/link-preview/badge.svg)](https://docs.rs/url-preview)
![Build](https://github.com/ZhangHanDong/url-preview/workflows/build/badge.svg)
![Clippy](https://github.com/ZhangHanDong/url-preview/workflows/clippy/badge.svg)
![Formatter](https://github.com/ZhangHanDong/url-preview/workflows/fmt/badge.svg)
![Tests](https://github.com/ZhangHanDong/url-preview/workflows/test/badge.svg)

</div>

# URL Preview

A high-performance Rust library for generating rich URL previews with specialized support for Twitter/X and GitHub. This library offers efficient caching, concurrent processing, and comprehensive metadata extraction capabilities.

## Features

- **High Performance**: Optimized for speed with concurrent processing and batch operations
- **Smart Caching**: Built-in DashMap-based caching system for lightning-fast responses
- **Platform-Specific Handlers**:
  - Twitter/X: Specialized handling with oEmbed support
  - GitHub: Enhanced repository information extraction
- **Flexible Configuration**:
  - Customizable HTTP clients with different configurations
  - Adjustable concurrent request limits
  - Configurable cache sizes
- **Rich Metadata Extraction**:
  - Title, description, and images
  - Open Graph and Twitter Card metadata
  - Favicons and site information
- **Robust Error Handling**:
  - Structured error types
  - Detailed logging with tracing support
  - Rate limiting protection
- **Modern Rust Features**:
  - Async/await with Tokio
  - Thread-safe components
  - Zero-cost abstractions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
url_preview = "0.1.0"
```

## Quick Start

Here's a simple example to get started:

```rust
use url_preview::{PreviewService, Preview, PreviewError};

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    // Create a preview service with default settings
    let preview_service = PreviewService::default();

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

### Batch Processing

Process multiple URLs efficiently:

```rust
use url_preview::{PreviewService, Fetcher};

let service = PreviewService::default();
let urls = vec![
    "https://www.rust-lang.org",
    "https://github.com/rust-lang/rust"
];

// Using batch fetching
let results = service.default_generator.fetcher
    .fetch_batch(urls)
    .await?;

// Or using concurrent processing
let results = futures::future::join_all(
    urls.iter().map(|url|
        service.generate_preview_with_concurrency(url)
    )
).await;
```

### Custom Configuration

Configure the service with specific requirements:

```rust
use url_preview::{PreviewService, PreviewServiceConfig, Fetcher, FetcherConfig};
use std::time::Duration;

let config = PreviewServiceConfig::new(1000) // Cache capacity
    .with_default_fetcher(
        Fetcher::new_with_config(FetcherConfig {
            timeout: Duration::from_secs(30),
            user_agent: "custom-agent/1.0".into(),
            ..Default::default()
        })
    );

let service = PreviewService::new_with_config(config);
```

### Specialized Platform Support

#### Twitter/X Integration

```rust
let preview = service
    .generate_preview("https://x.com/username/status/123456789")
    .await?;

// Automatic oEmbed support
println!("Tweet content: {:?}", preview.description);
```

#### GitHub Repository Information

```rust
let preview = service
    .generate_preview("https://github.com/owner/repo")
    .await?;

// Get detailed repository information
let details = service
    .get_github_detailed_info("https://github.com/owner/repo")
    .await?;

println!("Stars: {}", details.stars_count);
println!("Forks: {}", details.forks_count);
```

### Logging Configuration

Configure comprehensive logging:

```rust
use url_preview::{setup_logging, LogConfig};
use std::path::PathBuf;

let log_config = LogConfig {
    log_dir: PathBuf::from("logs"),
    log_level: "info".into(),
    console_output: true,
    file_output: true,
};

setup_logging(log_config);
```

## Performance Optimization

### Concurrent Request Limiting

Control the number of concurrent requests:

```rust
let config = PreviewServiceConfig {
    max_concurrent_requests: 10,
    cache_capacity: 1000,
    ..Default::default()
};

let service = PreviewService::new_with_config(config);
```

### Caching Strategy

The library uses DashMap for thread-safe, high-performance caching:

```rust
let cache = Cache::new(1000); // Configure cache size

// Cache operations are automatic in PreviewService
// Manual cache operations if needed:
cache.set("key".to_string(), preview).await;
let cached_preview = cache.get("key").await;
```

## Error Handling

The library provides comprehensive error handling:

```rust
match service.generate_preview(url).await {
    Ok(preview) => {
        println!("Successfully generated preview");
    }
    Err(PreviewError::RateLimitError(msg)) => {
        println!("Rate limit exceeded: {}", msg);
    }
    Err(PreviewError::FetchError(msg)) => {
        println!("Failed to fetch content: {}", msg);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`
4. Run benchmarks: `cargo bench`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Testing Websites for Rich Results

Google provides the [Rich Results Analysis Tool](https://search.google.com/test/rich-results?utm_source=support.google.com/webmasters/&utm_medium=referral&utm_campaign=7445569) to help you validate your website's tags.

Use this tool to make sure the website follows these conventions.
