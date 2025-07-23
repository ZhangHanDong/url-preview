# URL Preview Documentation

Welcome to the URL Preview library documentation! This directory contains comprehensive guides and references for using and contributing to the library.

## 📚 Documentation Index

### Getting Started
- **[Quick Start Guide](quickstart.md)** - Get up and running in minutes
- **[Feature Comparison](feature_comparison.md)** - Choose the right features for your needs
- **[Installation Guide](../README.md#installation)** - Detailed installation instructions

### Current Status
- **[Project Status](project_status.md)** - Current implementation status and roadmap
- **[CHANGELOG](../CHANGELOG.md)** - Version history and updates

### Development
- **[Testing Guide](testing_guide.md)** - How to test the library
- **[Architecture Overview](project_status.md#project-structure)** - Understanding the codebase
- **[Contributing Guidelines](../CONTRIBUTING.md)** - How to contribute

### API Reference
- **[API Documentation](https://docs.rs/url-preview)** - Full API reference on docs.rs
- **[Examples](../examples/)** - Working code examples

## 🚀 Quick Links

### By Feature
- **Basic Usage**: See [quickstart.md](quickstart.md#basic-usage)
- **Browser Rendering**: See [project_status.md#browser-rendering-implemented)
- **LLM Integration**: See [project_status.md#llm-integration-implemented)
- **Caching**: See [quickstart.md#2-with-caching)

### By Use Case
- **SPA Support**: Enable `browser` feature
- **AI Extraction**: Enable `llm` feature
- **Social Media**: Enable `browser` + platform features
- **Performance**: Enable `cache` feature

## 📋 Feature Overview

| Feature | Description | Use Case |
|---------|-------------|----------|
| `cache` | In-memory caching with DashMap | High-traffic applications |
| `browser` | JavaScript rendering via Playwright | SPAs and dynamic sites |
| `llm` | AI-powered data extraction | Structured data needs |
| `github` | GitHub API integration | Repository information |
| `twitter` | Twitter oEmbed support | Rich tweet previews |
| `logging` | Structured logging with tracing | Debugging and monitoring |

## 🛠️ Development Status

- **Core**: ✅ Stable
- **Browser**: ✅ Implemented (MCP + Playwright)
- **LLM**: ✅ Implemented (OpenAI + extensible)
- **Platforms**: ✅ GitHub, Twitter/X

See [project_status.md](project_status.md) for detailed status.

## 💡 Common Tasks

### Generate a Simple Preview
```rust
let service = PreviewService::new(Default::default());
let preview = service.generate_preview(url).await?;
```

### Render JavaScript Sites
```rust
let browser = BrowserPreviewService::new(config, BrowserUsagePolicy::Auto);
let preview = browser.generate_preview(spa_url).await?;
```

### Extract Structured Data
```rust
let extractor = LLMExtractor::new(provider);
let data = extractor.extract::<CustomType>(url, &fetcher).await?;
```

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/ZhangHanDong/url-preview/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ZhangHanDong/url-preview/discussions)
- **Examples**: [examples/](../examples/)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.