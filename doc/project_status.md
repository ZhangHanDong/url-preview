# URL Preview Library - Project Status

## Overview

URL Preview is a high-performance Rust library for generating rich URL previews with support for:
- Standard metadata extraction (Open Graph, Twitter Cards, etc.)
- Browser-based rendering for JavaScript-heavy sites
- LLM-powered structured data extraction
- Platform-specific handlers (Twitter/X, GitHub)
- Comprehensive caching and concurrency control

## Current Version: 0.6.0

## Feature Status

### ✅ Core Features (Stable)

1. **Basic Preview Generation**
   - HTML fetching and parsing
   - Metadata extraction (title, description, images)
   - Open Graph and Twitter Card support
   - Favicon detection
   - Character encoding handling

2. **Caching System**
   - Thread-safe DashMap-based cache
   - Configurable capacity and TTL
   - LRU eviction policy
   - Feature flag: `cache`

3. **Security & Validation**
   - URL validation
   - Content size limits
   - Request timeouts
   - Redirect following with limits

4. **Platform Handlers**
   - GitHub: Repository statistics via API
   - Twitter/X: oEmbed API integration
   - Feature flags: `github`, `twitter`

### ✅ Browser Rendering (Implemented)

**Status**: Fully implemented with MCP (Model Context Protocol) integration

**Components**:
- `McpClient`: Complete JSON-RPC communication layer
- `BrowserFetcher`: High-level browser automation
- `BrowserPreviewService`: Integrated preview generation

**Key Features**:
- Automatic SPA detection
- JavaScript rendering support
- Browser usage policies (Always/Never/Auto)
- Playwright integration via MCP
- Screenshot capabilities

**Example Usage**:
```rust
let mcp_config = McpConfig {
    enabled: true,
    server_command: vec!["npx", "-y", "@playwright/mcp@latest"],
    transport: McpTransport::Stdio,
    browser_timeout: 30,
    max_sessions: 5,
};

let browser_service = BrowserPreviewService::new(mcp_config, BrowserUsagePolicy::Auto);
browser_service.initialize().await?;
let preview = browser_service.generate_preview(url).await?;
```

**Run Examples**:
```bash
cargo run --example browser_preview --features browser
cargo run --example test_browser_simple --features browser
```

### ✅ LLM Integration (Implemented)

**Status**: Core functionality implemented with provider system

**Providers**:
- ✅ `MockProvider`: For testing
- ✅ `OpenAIProvider`: Full implementation with function calling
- 🚧 `AnthropicProvider`: Placeholder (awaiting official SDK)
- ✅ `ClaudeCompatProvider`: Claude via OpenAI-compatible endpoints
- ✅ `ClaudeCodeProvider`: Claude via claude-code-sdk (pending SDK fix)
- 🚧 `LocalProvider`: Placeholder for Ollama/local models

**Key Features**:
- Type-safe structured data extraction
- Automatic JSON schema generation
- Multiple content formats (HTML, Markdown, Text)
- Token usage tracking
- Optional caching support

**Example Usage**:
```rust
#[derive(Serialize, Deserialize, JsonSchema)]
struct ProductInfo {
    name: String,
    price: Option<String>,
    description: String,
}

let provider = Arc::new(OpenAIProvider::new(api_key));
let extractor = LLMExtractor::new(provider);
let result = extractor.extract::<ProductInfo>(url, &fetcher).await?;
```

**Run Examples**:
```bash
# With mock provider
cargo run --example test_llm_extraction --features "llm cache"

# With OpenAI
OPENAI_API_KEY=your_key cargo run --example test_llm_extraction --features "llm cache"

# Combined with browser
cargo run --example test_llm_with_browser --features "browser llm"
```

### 📝 Claude API Support

Claude can be used through multiple methods:

1. **Claude Code SDK** (Direct CLI integration):
   ```rust
   let provider = Arc::new(ClaudeCodeProvider::new());
   let extractor = LLMExtractor::new(provider);
   ```
   Note: Requires fixing a compilation issue in claude-code-sdk

2. **OpenRouter** (Recommended for API access):
   ```bash
   OPENROUTER_API_KEY=your_key cargo run --example claude_via_openrouter --features llm
   ```

3. **OpenAI-compatible proxies**: Helicone, LiteLLM, One API

4. **ClaudeCompatProvider**: Built-in provider for OpenAI-compatible endpoints

See `doc/claude_code_integration.md` for detailed integration guide.

## Project Structure

```
url-preview/
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── preview_service.rs     # High-level service interface
│   ├── fetcher.rs            # HTTP client wrapper
│   ├── extractor.rs          # Metadata extraction logic
│   ├── cache.rs              # Caching implementation
│   ├── browser_fetcher.rs    # Browser automation
│   ├── mcp_client.rs         # MCP protocol implementation
│   ├── llm_extractor.rs      # LLM extraction logic
│   └── llm_providers.rs      # LLM provider implementations
├── examples/
│   ├── browser_preview.rs    # Browser rendering demo
│   ├── test_llm_extraction.rs # LLM extraction demo
│   └── ...                   # Many more examples
├── tests/
│   ├── browser_tests.rs      # Browser integration tests
│   ├── llm_tests.rs         # LLM extraction tests
│   └── ...                  # Comprehensive test suite
└── doc/
    ├── testing_guide.md      # Testing documentation
    └── project_status.md     # This file
```

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `scraper`: HTML parsing
- `url`: URL parsing
- `serde`/`serde_json`: Serialization

### Optional Dependencies
- `dashmap`: Cache implementation (`cache` feature)
- `jsonrpc-core`: MCP protocol (`browser` feature)
- `async-openai`: OpenAI integration (`llm` feature)
- `schemars`: JSON schema generation (`llm` feature)
- `base64`: Screenshot decoding (`browser` feature)

## Testing

### Unit Tests
```bash
cargo test
```

### Feature-Specific Tests
```bash
# Browser features
cargo test --features browser

# LLM features
cargo test --features llm

# All features
cargo test --features full
```

### Integration Tests
- Browser tests require Node.js and npx
- LLM tests can run with mock provider or real API keys

## Known Limitations

1. **Browser Feature**:
   - Requires Node.js and npx installed
   - First run downloads Playwright browsers
   - Some sites may have anti-automation measures

2. **LLM Feature**:
   - OpenAI provider requires API key
   - Anthropic provider is placeholder only
   - Token limits apply based on model

3. **General**:
   - No streaming support for large responses
   - Limited multi-language support
   - No built-in rate limiting

## Future Enhancements

1. **Browser Improvements**:
   - Cookie/session management
   - Custom viewport sizes
   - Network request interception
   - Performance metrics

2. **LLM Enhancements**:
   - Native Anthropic SDK integration
   - Streaming response support
   - Local model support (Ollama)
   - Multi-modal extraction (with screenshots)

3. **Core Features**:
   - WebAssembly support
   - Batch processing optimization
   - Enhanced error recovery
   - Metrics and monitoring

## Performance Considerations

- Concurrent request limit: Configurable (default 10)
- Cache capacity: Configurable (default 1000 entries)
- Browser timeout: 30 seconds default
- LLM timeout: Based on model and content size

## API Stability

- Core preview API: Stable
- Browser API: Stable
- LLM API: Stable (providers may change)
- Internal modules: Subject to change

## Contributing

See CONTRIBUTING.md for guidelines. Key areas for contribution:
- Additional LLM provider implementations
- Platform-specific extractors
- Performance optimizations
- Documentation improvements

## License

MIT License - See LICENSE file for details.