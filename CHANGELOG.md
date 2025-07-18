# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
