# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2025-07-20

### 🚀 Features
- **Browser-based Preview Generation**: Added integration with Microsoft's playwright-mcp for JavaScript-rendered content
  - **MCP Client Integration**: 
    - Connects to playwright-mcp server via Model Context Protocol
    - Supports stdio and HTTP+SSE transports
    - Automatic browser lifecycle management
  - **Smart Browser Detection**:
    - Automatically detects SPAs and JavaScript-heavy sites
    - Configurable browser usage policies (Always/Never/Auto)
    - Built-in detection for popular platforms (Twitter, Reddit, Instagram, etc.)
  - **Browser Operations**:
    - Page navigation with configurable timeouts
    - JavaScript execution for custom data extraction
    - Screenshot capture for visual previews
    - Full DOM access after JavaScript rendering

- **LLM-powered Structured Data Extraction**: Extract structured data from web pages using Large Language Models
  - **Multiple Provider Support**:
    - **OpenAI Provider**: Full support for GPT-4, GPT-4o, GPT-3.5-turbo with function calling
    - **Anthropic Provider**: Complete native Claude API integration (Claude-3 series)
    - **claude-code-api Provider**: OpenAI-compatible interface for local Claude CLI (no API key needed)
    - **Local Provider**: Support for Ollama and other local model servers
    - **Mock Provider**: Full-featured testing provider with realistic responses
  - **Advanced Content Preprocessing**:
    - **Smart HTML Cleaning**: Removes scripts, styles, navigation elements while preserving content structure
    - **Multiple Format Support**: HTML, Markdown, and plain text output with intelligent conversion
    - **Content Optimization**: Configurable length limits and smart truncation for token efficiency
  - **Configuration & Automation**:
    - **Auto-detection**: Intelligent provider selection based on available API keys and services
    - **Environment-based Setup**: Automatic configuration from environment variables
    - **API Key Validation**: Built-in validation for OpenAI (`sk-*`) and Anthropic (`sk-ant-*`) keys
    - **Model Validation**: Ensures valid model names for each provider
  - **Response Format Compatibility**:
    - **Dual Format Support**: Handles both OpenAI function calling and plain text JSON responses
    - **Intelligent Parsing**: Automatically extracts JSON from text responses for claude-code-api compatibility
    - **Error Recovery**: Graceful fallback mechanisms for different response formats
  - **Schema-driven Extraction**:
    - **Type-safe Extraction**: Use Rust structs with derive macros for extraction schemas
    - **JSON Schema Generation**: Automatic schema generation with schemars crate
    - **Rich Result Types**: Detailed extraction results with token usage tracking
    - **Comprehensive Error Handling**: Specific error types for different failure scenarios

### 🛠️ New Types and APIs
- Browser support types:
  - `McpClient`: MCP protocol client for browser automation
  - `McpConfig`: Configuration for MCP integration
  - `BrowserFetcher`: Browser-based content fetcher
  - `BrowserPreviewService`: High-level browser preview service
  - `BrowserUsagePolicy`: Control when to use browser (Always/Never/Auto)

- LLM extraction types:
  - `LLMExtractor`: Main extractor for structured data extraction
  - `LLMExtractorConfig`: Comprehensive configuration for extraction behavior
  - `LLMProvider`: Trait for implementing custom LLM providers
  - `ContentFormat`: Enum for content preprocessing formats (Html, Markdown, Text, Image)
  - `ExtractionResult<T>`: Type-safe extraction results with usage tracking
  - `ProcessedContent`: Preprocessed and optimized content ready for LLM consumption
  - `TokenUsage`: Token consumption tracking for cost monitoring

- LLM provider implementations:
  - `OpenAIProvider`: OpenAI API integration with function calling support
  - `AnthropicProvider`: Native Anthropic Claude API integration
  - `LocalProvider`: Support for Ollama and local model servers
  - `MockProvider`: Full-featured testing provider

- Configuration and validation:
  - `LLMConfig`: Helper for environment-based provider configuration
  - `ApiKeyValidator`: Validation utilities for API keys and model names
  - `InvalidConfiguration`: New error type for configuration issues

### 📚 Examples
- **Browser Integration**:
  - `browser_preview.rs`: Basic browser-based preview generation
  - `browser_llm_extraction.rs`: Combined browser + LLM extraction

- **LLM-powered Extraction**:
  - `comprehensive_llm_test.rs`: Complete demonstration of all LLM features
  - `test_anthropic_direct.rs`: Direct Anthropic API usage examples
  - `test_openai_extraction.rs`: OpenAI API integration and best practices
  - `claude_api_working.rs`: claude-code-api integration with compatibility fixes
  - `llm_extraction.rs`: Basic LLM data extraction patterns
  - `test_json_extraction.rs`: JSON parsing compatibility tests
  - `custom_claude_provider.rs`: Custom provider implementation example

### 🐛 Bug Fixes & Improvements
- **LLM Integration Fixes**:
  - Fixed all compilation errors in LLM-related code
  - Resolved dead code warnings and unused field issues
  - Fixed error type usage to match current PreviewError enum structure
  - Implemented proper JSON extraction from text responses for claude-code-api compatibility

- **HTML Processing Enhancements**:
  - Upgraded HTML cleaner from placeholder to full implementation
  - Intelligent content structure preservation during cleaning
  - Smart removal of unnecessary elements (scripts, styles, navigation)
  - Improved text extraction with proper whitespace handling

- **Error Handling Improvements**:
  - Added `InvalidConfiguration` error type for better configuration validation
  - More specific error messages with context information
  - Proper error propagation throughout the LLM pipeline
  - Graceful fallback mechanisms for different response formats

### 🧪 Testing
- **Comprehensive Test Coverage**:
  - `browser_tests.rs`: Unit tests for browser functionality
  - `llm_tests.rs`: Tests for LLM extraction features
  - Configuration validation tests for all providers
  - JSON extraction compatibility tests
  - Error handling scenario tests
- All features are behind feature flags for modular compilation
- Real-world integration tests with actual API responses

### 🔧 Configuration
- New feature flags:
  - `browser`: Enable browser-based fetching (requires Node.js)
  - `llm`: Enable LLM extraction capabilities
- Enhanced `PreviewServiceConfig` with browser and LLM options

### 💥 Breaking Changes
- None - all new features are additive and behind feature flags

### 📦 Dependencies
- **Browser Integration**:
  - `jsonrpc-core`: JSON-RPC protocol support for MCP communication
  - `jsonrpc-stdio-server`: Stdio transport layer for MCP
  - `base64`: Binary data encoding for browser operations

- **LLM Integration**:
  - `schemars`: JSON Schema generation from Rust structs
  - `async-openai`: OpenAI API client with function calling support
  - `cc-sdk`: Claude Code SDK integration (optional, path dependency)

- **Enhanced Processing**:
  - `scraper`: HTML parsing and manipulation for content cleaning
  - `std::collections::HashMap`: Enhanced data structures for content metadata

## [0.5.0] - 2025-07-20

### 🚀 Features
- **Comprehensive Security Framework**: Added complete security features to protect against common web scraping vulnerabilities
  - **URL Validation**: 
    - Scheme validation (default: http/https only)
    - Private IP blocking (RFC 1918 ranges)
    - Localhost blocking (127.0.0.1, ::1, localhost)
    - Domain whitelist/blacklist support
    - Configurable redirect limits
  - **Content Security**:
    - Content size limits (default: 10MB)
    - Download time limits (default: 30s)
    - Content type filtering (default: HTML/JSON only)
    - Streaming validation to prevent memory exhaustion
  - **SSRF Protection**: Complete protection against Server-Side Request Forgery attacks
    - IPv4 private ranges: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, etc.
    - IPv6 private ranges: fe80::/10, fc00::/7
    - Special IP ranges: multicast, reserved, broadcast

### 🛡️ Security Types
- Added new security-focused types:
  - `UrlValidator`: Validates URLs against security policies
  - `UrlValidationConfig`: Configurable URL validation rules
  - `ContentLimits`: Content size and time restrictions
  - New error variants: `InvalidUrlScheme`, `DomainBlocked`, `LocalhostBlocked`, `PrivateIpBlocked`, etc.

### 📚 Documentation
- Added comprehensive security documentation (`doc/security.md`)
- Updated codebase analysis with security implementation details
- Added security-focused examples:
  - `security_validation.rs`: URL validation examples
  - `content_limits.rs`: Content restriction examples
  - `secure_preview_cli.rs`: Full CLI with security options

### 🧪 Testing
- Added extensive security test suite:
  - `security_tests.rs`: Unit tests for all security features
  - `security_integration_tests.rs`: Integration tests for security scenarios
- All security features are thoroughly tested

### 💥 Breaking Changes
- `FetcherConfig` now includes security configuration fields:
  - `url_validation: UrlValidationConfig`
  - `content_limits: ContentLimits`
- Default behavior now blocks potentially dangerous URLs (localhost, private IPs)
- To restore previous permissive behavior, explicitly configure security settings

### 🔧 Configuration
- Security can be customized via `FetcherConfig`:
  ```rust
  let config = FetcherConfig {
      url_validation: UrlValidationConfig {
          block_private_ips: false,  // Allow private IPs
          block_localhost: false,    // Allow localhost
          // ... other settings
      },
      content_limits: ContentLimits {
          max_content_size: 20 * 1024 * 1024,  // 20MB
          max_download_time: 60,                // 60 seconds
          // ... other settings
      },
      ..Default::default()
  };
  ```

## [0.4.0] - 2025-07-18

### 🚀 Features
- **Enhanced Error Handling**: Added specific error types for better error differentiation
  - `DnsError`: For DNS resolution failures
  - `ConnectionError`: For connection issues
  - `HttpError`: Generic HTTP errors with status codes
  - `ServerError`: Specific for 5xx errors
  - `ClientError`: Specific for 4xx errors
- **Smart Error Conversion**: Added `PreviewError::from_reqwest_error()` for intelligent error mapping
- **Invalid URL Detection**: Properly detect and report 404s and invalid resources
  - GitHub repositories now return `NotFound` for non-existent repos
  - Twitter/X posts return appropriate errors for deleted or invalid tweets
  - General URLs return `NotFound` for 404 responses

### 🚜 Refactor
- Restructured fetcher module for better error handling and performance
- Improved preview_service concurrent request handling
- Optimized cache operations and processing flow
- Updated examples to handle feature flags properly

### 🐛 Bug Fixes
- Fixed compilation issues with optional features in examples and benchmarks
- Fixed test failures due to improved error handling
- Properly handle HTTP status codes throughout the codebase
- Fixed feature-gated code compilation issues

### 💥 Breaking Changes
- Error handling has been significantly improved. Applications using `PreviewError::FetchError` will need to handle new specific error variants:
  - `DnsError`, `ConnectionError`, `HttpError`, `ServerError`, `ClientError`
- The generic `FetchError` is now only used as a fallback for unspecified errors

### 📚 Documentation
- Updated README with comprehensive examples of new error handling
- Added examples for handling different error types
- Improved documentation for all public APIs

## [0.3.2] - Previous Release

### Features
- Basic URL preview generation
- Twitter/X oEmbed support
- GitHub repository preview support
- Caching with DashMap
- Concurrent request processing

## [0.3.0] - Earlier Release

### Features
- Initial implementation of core functionality
- Basic metadata extraction
- Simple caching system

---

For more details, see the [GitHub releases](https://github.com/ZhangHanDong/url-preview/releases).
