# URL Preview Library - Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
url-preview = { version = "0.6.0", features = ["full"] }
```

Or select specific features:

```toml
[dependencies]
url-preview = { version = "0.6.0", features = ["cache", "browser", "llm"] }
```

## Basic Usage

### 1. Simple Preview Generation

```rust
use url_preview::{PreviewService, PreviewServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service with default config
    let service = PreviewService::new(PreviewServiceConfig::default());
    
    // Generate preview
    let preview = service.generate_preview("https://rust-lang.org").await?;
    
    println!("Title: {:?}", preview.title);
    println!("Description: {:?}", preview.description);
    println!("Image: {:?}", preview.image_url);
    
    Ok(())
}
```

### 2. With Caching

```rust
use url_preview::{PreviewService, PreviewServiceConfig};

let config = PreviewServiceConfig::new(1000) // Cache capacity
    .with_timeout(30);

let service = PreviewService::new(config);

// First call - fetches from web
let preview1 = service.generate_preview(url).await?;

// Second call - returns from cache
let preview2 = service.generate_preview(url).await?;
```

### 3. Browser Rendering (JavaScript Sites)

```rust
use url_preview::{
    BrowserPreviewService, McpConfig, McpTransport, BrowserUsagePolicy
};

// Configure browser
let mcp_config = McpConfig {
    enabled: true,
    server_command: vec![
        "npx".to_string(),
        "-y".to_string(),
        "@playwright/mcp@latest".to_string(),
    ],
    transport: McpTransport::Stdio,
    browser_timeout: 30,
    max_sessions: 5,
};

// Create service
let browser_service = BrowserPreviewService::new(
    mcp_config, 
    BrowserUsagePolicy::Auto // Auto-detect SPAs
);

// Initialize (starts browser)
browser_service.initialize().await?;

// Generate preview with browser rendering
let preview = browser_service.generate_preview("https://twitter.com/rustlang").await?;
```

### 4. LLM-Powered Extraction

```rust
use url_preview::{
    Fetcher, LLMExtractor, OpenAIProvider, LLMProvider
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

// Define what to extract
#[derive(Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    author: Option<String>,
    summary: String,
    topics: Vec<String>,
}

// Setup LLM
let provider = Arc::new(OpenAIProvider::new("your-api-key"));
let extractor = LLMExtractor::new(provider);
let fetcher = Arc::new(Fetcher::new());

// Extract structured data
let result = extractor.extract::<ArticleInfo>(
    "https://blog.rust-lang.org/",
    &fetcher
).await?;

println!("Title: {}", result.data.title);
println!("Topics: {:?}", result.data.topics);
```

## Feature Combinations

### Browser + LLM for Dynamic Sites

```rust
// 1. Render with browser
let browser_service = BrowserPreviewService::new(config, BrowserUsagePolicy::Always);
browser_service.initialize().await?;

// 2. Get rendered HTML
let preview = browser_service.generate_preview(url).await?;

// 3. Extract structured data with LLM
let llm_extractor = LLMExtractor::new(provider);
let data = llm_extractor.extract::<ProductInfo>(url, &fetcher).await?;
```

## Common Patterns

### 1. Batch Processing

```rust
use futures::future::join_all;

let urls = vec![
    "https://example1.com",
    "https://example2.com",
    "https://example3.com",
];

let futures = urls.iter().map(|url| {
    service.generate_preview(url)
});

let results = join_all(futures).await;
```

### 2. Error Handling

```rust
use url_preview::PreviewError;

match service.generate_preview(url).await {
    Ok(preview) => {
        // Handle success
    }
    Err(PreviewError::NetworkError(e)) => {
        // Handle network issues
    }
    Err(PreviewError::InvalidUrl(url)) => {
        // Handle invalid URLs
    }
    Err(e) => {
        // Handle other errors
    }
}
```

### 3. Custom Configuration

```rust
use url_preview::{PreviewServiceConfig, FetcherConfig};
use std::time::Duration;

let config = PreviewServiceConfig::new(500)
    .with_timeout(60)
    .with_max_redirects(5)
    .with_user_agent("MyApp/1.0");

let service = PreviewService::new(config);
```

## Platform-Specific Examples

### Twitter/X Preview

```rust
// Automatic Twitter handling with oEmbed
let preview = service.generate_preview("https://twitter.com/rustlang/status/123").await?;

// Returns rich tweet data when twitter feature is enabled
```

### GitHub Repository

```rust
// Automatic GitHub API integration
let preview = service.generate_preview("https://github.com/rust-lang/rust").await?;

// Returns repository stats when github feature is enabled
```

## Environment Variables

```bash
# For LLM features
export OPENAI_API_KEY=sk-...

# For Claude via OpenRouter
export OPENROUTER_API_KEY=sk-or-...

# For logging
export RUST_LOG=url_preview=debug
```

## Running Examples

```bash
# Basic preview
cargo run --example url_preview

# Browser rendering
cargo run --example browser_preview --features browser

# LLM extraction
cargo run --example test_llm_extraction --features llm

# All features
cargo run --example test_all_features --features full
```

## Troubleshooting

### Browser Issues
- Ensure Node.js and npm are installed
- First run downloads Playwright browsers
- Check firewall settings for `npx`

### LLM Issues
- Verify API key is set correctly
- Check API rate limits
- Ensure sufficient token quota

### Performance
- Enable caching for repeated URLs
- Use appropriate concurrency limits
- Consider browser usage policy

## Next Steps

- See [examples/](../examples/) for more code samples
- Read [API documentation](https://docs.rs/url-preview)
- Check [project status](project_status.md) for feature details
- Report issues on [GitHub](https://github.com/ZhangHanDong/url-preview)