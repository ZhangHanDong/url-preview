# Claude Code SDK Integration Guide

This document describes how to integrate the `rust-claude-code-api` project with `url-preview` for LLM-based structured data extraction.

## Overview

The integration allows you to use Claude Code CLI as an LLM provider for extracting structured data from web pages. This provides a powerful alternative to OpenAI or other API-based providers.

## Implementation Status

### Completed ✅

1. **LLM Provider Implementation**: Created `ClaudeCodeProvider` in `src/llm_providers/claude_code.rs`
   - Implements the `LLMProvider` trait
   - Supports all Claude 3 models (Opus, Sonnet, Haiku)
   - Configurable system prompts and thinking tokens

2. **Cargo Configuration**: Updated `Cargo.toml` with optional dependency
   - Added `claude-code-sdk` as an optional dependency
   - Created `claude-code` feature flag for conditional compilation

3. **Module Exports**: Updated `src/lib.rs` to export `ClaudeCodeProvider`
   - Conditional export based on `claude-code` feature flag

4. **Example Code**: Created multiple examples demonstrating usage
   - `test_claude_code_integration.rs`: Comprehensive integration example
   - `simple_claude_code_example.rs`: Simple usage example
   - `test_claude_code_mock.rs`: Mock implementation for testing

### Pending 🚧

1. **SDK Compilation Issue**: The `claude-code-sdk` has a borrowing error that needs to be fixed:
   ```
   error[E0499]: cannot borrow `transport` as mutable more than once at a time
   ```
   This is in the SDK's `query.rs` file and needs to be resolved upstream.

## Usage

Once the SDK compilation issue is fixed, you can use the integration as follows:

### 1. Add Dependencies

```toml
[dependencies]
url-preview = { version = "0.6", features = ["llm", "claude-code"] }
```

### 2. Basic Usage

```rust
use url_preview::{LLMExtractor, ClaudeCodeProvider, Fetcher};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    key_points: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = std::sync::Arc::new(ClaudeCodeProvider::new());
    
    // Create fetcher
    let fetcher = Fetcher::new();
    
    // Create extractor
    let extractor = LLMExtractor::new(provider);
    
    // Extract data
    let url = "https://blog.rust-lang.org/";
    let result = extractor.extract::<ArticleInfo>(url, &fetcher).await?;
    
    println!("Title: {}", result.data.title);
    println!("Summary: {}", result.data.summary);
    
    Ok(())
}
```

### 3. With Custom Configuration

```rust
use url_preview::{LLMExtractorConfig, ContentFormat};

// Create custom provider with Sonnet model
let provider = ClaudeCodeProvider::new()
    .with_sonnet()
    .with_system_prompt("You are an expert data analyst.")
    .with_max_thinking_tokens(10000);

// Create custom config
let config = LLMExtractorConfig {
    format: ContentFormat::Markdown,
    clean_html: true,
    max_content_length: 10000,
    model_params: Default::default(),
};

// Create extractor with custom config
let extractor = LLMExtractor::with_config(
    Arc::new(provider),
    config
);
```

## Architecture

```
┌─────────────────────┐
│   url-preview app   │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│    LLMExtractor     │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│ ClaudeCodeProvider  │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│  claude-code-sdk    │
└─────────┬───────────┘
          │
┌─────────▼───────────┐
│  Claude Code CLI    │
└─────────────────────┘
```

## Benefits

1. **Direct Claude Access**: No need for API keys or proxy services
2. **Tool Use Support**: Full access to Claude's tool-use capabilities
3. **Interactive Sessions**: Support for stateful conversations
4. **Local Development**: Works with local Claude Code CLI installation
5. **Cost Effective**: Uses your existing Claude Code subscription

## Prerequisites

1. Install Claude Code CLI:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```

2. Ensure you're logged in to Claude Code:
   ```bash
   claude-code --help
   ```

## Fixing the SDK Issue

To fix the compilation issue in `claude-code-sdk`, the `query.rs` file needs to be updated to properly handle the mutable borrow of `transport`. The issue is around line 168 where `transport.disconnect()` is called while `stream` still holds a mutable reference.

Possible fix:
```rust
// Drop the stream before disconnecting
drop(stream);
if let Err(e) = transport.disconnect().await {
    tracing::warn!("Failed to disconnect transport: {}", e);
}
```

## Testing

Run the mock example to see the integration pattern:
```bash
cargo run --example test_claude_code_mock --features llm
```

Once the SDK is fixed, run the full integration:
```bash
cargo run --example test_claude_code_integration --features "llm claude-code"
```

## Troubleshooting

1. **SDK Compilation Error**: Wait for upstream fix or apply the patch mentioned above
2. **Claude CLI Not Found**: Ensure Claude Code CLI is installed globally
3. **Authentication Issues**: Run `claude-code` directly to ensure you're logged in
4. **Feature Not Found**: Make sure to enable both `llm` and `claude-code` features