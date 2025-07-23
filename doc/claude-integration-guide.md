# Claude Integration Guide

This guide explains the two different ways to integrate Claude with url-preview.

## Overview

url-preview supports two methods for using Claude:

1. **cc-sdk**: Direct integration with Claude CLI (faster, simpler)
2. **claude-code-api**: HTTP API service (more flexible, service-oriented)

## Method 1: cc-sdk (Direct CLI Integration)

### Setup

1. Install Claude CLI:
   ```bash
   npm install -g @anthropic-ai/claude-cli
   ```

2. Authenticate:
   ```bash
   claude auth login
   ```

3. Enable the feature in `Cargo.toml`:
   ```toml
   [dependencies]
   url-preview = { version = "0.6", features = ["claude-code"] }
   ```

### Usage

```rust
use url_preview::{ClaudeCodeProvider, LLMExtractor, Fetcher};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    topics: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider with direct CLI integration
    let provider = Arc::new(
        ClaudeCodeProvider::new()
            .with_haiku()  // Fast model (aliases: "haiku", "sonnet", "opus")
    );
    
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    // Extract structured data
    let result = extractor.extract::<ArticleInfo>(
        "https://blog.rust-lang.org/",
        &fetcher
    ).await?;
    
    println!("Title: {}", result.data.title);
    println!("Topics: {:?}", result.data.topics);
    
    Ok(())
}
```

### Model Selection

```rust
// Default model (sonnet)
let provider = Arc::new(ClaudeCodeProvider::new());

// Fast model (haiku)
let provider = Arc::new(ClaudeCodeProvider::new().with_haiku());

// Most capable model (opus)
let provider = Arc::new(ClaudeCodeProvider::new().with_opus());

// Custom model by alias
let provider = Arc::new(
    ClaudeCodeProvider::new()
        .with_model("sonnet".to_string())
);
```

### Advantages
- ✅ No API key management
- ✅ Direct CLI calls (faster)
- ✅ Uses your Claude subscription
- ✅ Simple setup

### Disadvantages
- ❌ Requires Claude CLI installed locally
- ❌ Not suitable for serverless/container deployments
- ❌ Limited to Claude CLI capabilities

## Method 2: claude-code-api (HTTP Service)

### Setup

1. Install and run claude-code-api server:
   ```bash
   # Install
   npm install -g claude-code-api
   
   # Run server
   claude-code-api --port 8080
   ```

2. Enable the feature in `Cargo.toml`:
   ```toml
   [dependencies]
   url-preview = { version = "0.6", features = ["llm"] }
   ```

### Usage

```rust
use url_preview::{LLMConfig, LLMExtractor, Fetcher};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    topics: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-configure from environment
    let provider = LLMConfig::claude_code_from_env()?;
    
    // Or manually configure
    use url_preview::OpenAIProvider;
    use async_openai::config::OpenAIConfig;
    
    let openai_config = OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(
            openai_config, 
            "claude-3-5-haiku-20241022".to_string()
        )
    );
    
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    // Extract structured data
    let result = extractor.extract::<ArticleInfo>(
        "https://blog.rust-lang.org/",
        &fetcher
    ).await?;
    
    println!("Title: {}", result.data.title);
    
    Ok(())
}
```

### Advantages
- ✅ OpenAI-compatible API
- ✅ Can be deployed as a service
- ✅ Works in containers/serverless
- ✅ Can be shared across applications

### Disadvantages
- ❌ Requires running separate server
- ❌ Additional HTTP overhead
- ❌ More complex setup

## Comparison Table

| Feature | cc-sdk | claude-code-api |
|---------|--------|-----------------|
| **Setup Complexity** | Simple | Moderate |
| **Performance** | Faster (direct) | Slower (HTTP) |
| **Deployment** | Local only | Anywhere |
| **API Key** | Not needed | Not needed |
| **Feature Flag** | `claude-code` | `llm` |
| **Model Names** | Aliases only | Full names |
| **Error Handling** | Direct errors | HTTP errors |
| **Scalability** | Single instance | Multi-instance |
| **Best For** | Development | Production services |

## Error Handling

### cc-sdk Errors

```rust
match extractor.extract::<MyData>(url, &fetcher).await {
    Ok(result) => { /* success */ },
    Err(e) => {
        match e {
            PreviewError::ExternalServiceError { service, message } 
                if service == "Claude Code" => {
                // Claude CLI error - check auth or installation
            },
            PreviewError::ParseError(_) => {
                // JSON parsing failed
            },
            _ => { /* other errors */ }
        }
    }
}
```

### claude-code-api Errors

```rust
match extractor.extract::<MyData>(url, &fetcher).await {
    Ok(result) => { /* success */ },
    Err(e) => {
        match e {
            PreviewError::HttpError(status) => {
                // HTTP error from API server
            },
            PreviewError::ConnectionError(_) => {
                // Can't reach claude-code-api server
            },
            _ => { /* other errors */ }
        }
    }
}
```

## Testing

Both methods can be tested with the provided examples:

```bash
# Test cc-sdk
cargo run --example cc_sdk_simple_test --features claude-code
cargo run --example cc_sdk_working_test --features claude-code

# Compare both methods
cargo run --example claude_integration_comparison --features claude-code
```

## Recommendations

- **For Development**: Use cc-sdk for simplicity and speed
- **For Production**: Use claude-code-api for flexibility and scalability
- **For Testing**: Start with cc-sdk, migrate to claude-code-api when needed

## Troubleshooting

### cc-sdk Issues

1. **"Invalid model name"**: Use aliases ("haiku", "sonnet", "opus") not full names
2. **"Command not found"**: Install Claude CLI with `npm install -g @anthropic-ai/claude-cli`
3. **"Not authenticated"**: Run `claude auth login`

### claude-code-api Issues

1. **"Connection refused"**: Ensure server is running on correct port
2. **"Invalid API key"**: API key can be any string (not validated)
3. **"Model not found"**: Use full model names like "claude-3-5-haiku-20241022"