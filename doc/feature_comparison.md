# URL Preview Library - Feature Comparison

## Feature Matrix

| Feature | Basic | Cache | Browser | LLM | GitHub | Twitter | Full |
|---------|-------|-------|---------|-----|--------|---------|------|
| Basic metadata extraction | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Open Graph support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Twitter Cards | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Favicon detection | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| In-memory caching | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| JavaScript rendering | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| SPA support | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| Screenshot capture | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| Structured extraction | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| AI-powered analysis | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| GitHub API integration | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Twitter oEmbed API | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |

## Use Cases by Feature

### Default (No features)
```toml
url-preview = "0.6.0"
```
- Simple websites with static metadata
- Basic preview generation
- Minimal dependencies
- Fast and lightweight

### With Cache
```toml
url-preview = { version = "0.6.0", features = ["cache"] }
```
- High-traffic applications
- Repeated URL requests
- API rate limit management
- Performance optimization

### With Browser
```toml
url-preview = { version = "0.6.0", features = ["browser"] }
```
- Single Page Applications (SPAs)
- JavaScript-heavy sites
- Dynamic content rendering
- Sites requiring interaction

**Supported Sites**:
- Twitter/X
- Instagram
- LinkedIn
- Reddit
- Discord
- Modern web apps

### With LLM
```toml
url-preview = { version = "0.6.0", features = ["llm"] }
```
- Structured data extraction
- Content analysis
- Sentiment analysis
- Custom data schemas

**Providers**:
- OpenAI (GPT-4, GPT-3.5)
- Claude (via OpenRouter)
- Mock (for testing)

### Platform-Specific

#### GitHub
```toml
url-preview = { version = "0.6.0", features = ["github"] }
```
- Repository statistics
- Stars, forks, issues count
- Language detection
- Last update time

#### Twitter
```toml
url-preview = { version = "0.6.0", features = ["twitter"] }
```
- Rich tweet embeds
- Author information
- Engagement metrics
- Media previews

### Full Features
```toml
url-preview = { version = "0.6.0", features = ["full"] }
```
All capabilities combined for maximum flexibility.

## Performance Impact

| Feature | Binary Size | Memory Usage | Startup Time | Dependencies |
|---------|------------|--------------|--------------|--------------|
| Default | Baseline | Low | Fast | Minimal |
| +Cache | +100KB | Medium | Fast | +dashmap |
| +Browser | +500KB | High | Slow* | +jsonrpc, +base64 |
| +LLM | +1MB | Medium | Fast | +async-openai, +schemars |
| +GitHub | +50KB | Low | Fast | None |
| +Twitter | +50KB | Low | Fast | None |

*Browser startup requires downloading Playwright on first run

## Decision Guide

### Choose Default when:
- Building lightweight applications
- Only need basic previews
- Want minimal dependencies
- Performance is critical

### Add Cache when:
- Handling many repeated URLs
- Want to reduce network requests
- Need consistent response times
- Building high-traffic services

### Add Browser when:
- Targeting modern web apps
- Need JavaScript execution
- Dealing with SPAs
- Require screenshots

### Add LLM when:
- Need structured data
- Want intelligent extraction
- Building data pipelines
- Require content analysis

### Add Platform features when:
- Specifically targeting those platforms
- Need rich platform-specific data
- Want native API integration

## Feature Combinations

### Common Combinations

1. **Web Scraper**: `["browser", "llm", "cache"]`
   - Render dynamic content
   - Extract structured data
   - Cache results

2. **Social Media Aggregator**: `["browser", "twitter", "cache"]`
   - Handle social platforms
   - Rich embeds
   - Efficient caching

3. **Documentation Tool**: `["github", "cache"]`
   - Repository information
   - Fast repeated access
   - Low overhead

4. **Content Analyzer**: `["llm", "cache"]`
   - AI-powered extraction
   - Cached analysis
   - Cost optimization

## Migration Guide

### From Basic to Full Features

1. **Add Browser Support**:
   ```rust
   // Before
   let service = PreviewService::new(config);
   
   // After
   let browser_service = BrowserPreviewService::new(mcp_config, policy);
   browser_service.initialize().await?;
   ```

2. **Add LLM Extraction**:
   ```rust
   // Additional capability
   let extractor = LLMExtractor::new(provider);
   let data = extractor.extract::<YourType>(url, &fetcher).await?;
   ```

3. **Enable Caching**:
   ```rust
   // Just update config
   let config = PreviewServiceConfig::new(1000); // Cache size
   ```

## Cost Considerations

| Feature | Cost Factor | Mitigation |
|---------|------------|------------|
| Browser | CPU/Memory for Playwright | Use Auto policy |
| LLM | API calls (OpenAI/Claude) | Enable caching |
| GitHub | API rate limits | Cache responses |
| Twitter | API rate limits | Use oEmbed endpoint |

## Future Features

Planned additions:
- Video preview support
- PDF content extraction
- Image analysis
- WebAssembly runtime
- GraphQL API support
- Webhook notifications