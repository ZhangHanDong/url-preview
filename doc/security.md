# Security Features Documentation

## Overview

The `url-preview` library implements comprehensive security measures to protect against common web scraping vulnerabilities and ensure safe URL preview generation. This document details all security features available in version 0.5.0 and above.

## Table of Contents

1. [URL Validation](#url-validation)
2. [Content Security](#content-security)
3. [Network Security](#network-security)
4. [Error Handling](#error-handling)
5. [Usage Examples](#usage-examples)
6. [Best Practices](#best-practices)

## URL Validation

### Overview

The library provides robust URL validation through the `UrlValidator` class, which prevents access to potentially dangerous URLs and implements SSRF (Server-Side Request Forgery) protection.

### Features

#### 1. **Scheme Validation**
- Default allowed schemes: `http`, `https`
- Blocks dangerous schemes like `file://`, `ftp://`, `javascript:`, etc.
- Configurable through `UrlValidationConfig`

#### 2. **Private IP Blocking**
- Blocks access to private IP ranges (RFC 1918)
- Prevents SSRF attacks targeting internal networks
- Includes IPv4 ranges:
  - 10.0.0.0/8
  - 172.16.0.0/12
  - 192.168.0.0/16
  - 169.254.0.0/16 (link-local)
  - 100.64.0.0/10 (carrier-grade NAT)
- Includes IPv6 ranges:
  - fe80::/10 (link-local)
  - fc00::/7 (unique local)

#### 3. **Localhost Blocking**
- Blocks `localhost`, `127.0.0.1`, `::1`
- Prevents access to local services

#### 4. **Domain Filtering**
- **Blacklist**: Block specific domains or subdomains
- **Whitelist**: Only allow specific domains (when configured)
- Subdomain matching included

#### 5. **Redirect Control**
- Configurable maximum redirect count (default: 10)
- Prevents redirect loops and excessive redirections

### Configuration

```rust
use url_preview::{UrlValidationConfig, UrlValidator};
use std::collections::HashSet;

let mut config = UrlValidationConfig::default();

// Customize allowed schemes
config.allowed_schemes.insert("https".to_string());
config.allowed_schemes.remove("http"); // HTTPS only

// Add blocked domains
config.blocked_domains.insert("malicious.com".to_string());
config.blocked_domains.insert("phishing.site".to_string());

// Or use whitelist mode
config.allowed_domains.insert("trusted.com".to_string());
config.allowed_domains.insert("mycompany.com".to_string());

// Adjust security settings
config.block_private_ips = true;
config.block_localhost = true;
config.max_redirects = 5;

let validator = UrlValidator::new(config);
```

## Content Security

### Overview

The library implements content security measures to prevent resource exhaustion and ensure safe content processing.

### Features

#### 1. **Content Size Limits**
- Default maximum: 10MB
- Prevents memory exhaustion from large files
- Enforced both via Content-Length header and streaming

#### 2. **Download Time Limits**
- Default maximum: 30 seconds
- Prevents slow-loris style attacks
- Ensures timely response

#### 3. **Content Type Filtering**
- Default allowed types:
  - `text/html`
  - `application/xhtml+xml`
  - `text/plain`
  - `application/json`
- Prevents downloading binary files or unexpected content

### Configuration

```rust
use url_preview::{ContentLimits, FetcherConfig};
use std::collections::HashSet;

let mut content_limits = ContentLimits::default();

// Adjust size limit (5MB)
content_limits.max_content_size = 5 * 1024 * 1024;

// Adjust time limit (15 seconds)
content_limits.max_download_time = 15;

// Customize allowed content types
content_limits.allowed_content_types.clear();
content_limits.allowed_content_types.insert("text/html".to_string());
content_limits.allowed_content_types.insert("application/json".to_string());

let config = FetcherConfig {
    content_limits,
    ..Default::default()
};
```

## Network Security

### Built-in Features

#### 1. **Timeout Protection**
- Default request timeout: 10 seconds
- Twitter-specific timeout: 30 seconds
- Configurable per-request

#### 2. **Connection Pooling**
- Maximum 10 idle connections per host
- Prevents connection exhaustion

#### 3. **Retry Logic**
- Maximum 3 retries with exponential backoff
- Only retries on server errors (5xx) and timeouts
- Prevents amplification attacks

#### 4. **Concurrent Request Limiting**
- Default limit: 500 concurrent requests
- Prevents resource exhaustion
- Configurable via `PreviewServiceConfig`

### SSL/TLS Security

The library uses `reqwest`'s default TLS configuration, which includes:
- TLS 1.2+ support
- Certificate validation
- Hostname verification

## Error Handling

### Security-Specific Error Types

The library provides detailed error types for security-related issues:

```rust
pub enum PreviewError {
    // URL validation errors
    InvalidUrlScheme(String),
    InvalidUrl(String),
    DomainNotAllowed(String),
    DomainBlocked(String),
    LocalhostBlocked,
    PrivateIpBlocked(String),
    
    // Content security errors
    ContentSizeExceeded { size: usize, limit: usize },
    DownloadTimeExceeded { elapsed: u64, limit: u64 },
    ContentTypeNotAllowed(String),
    
    // Other security-relevant errors
    TimeoutError(String),
    DnsError(String),
    ConnectionError(String),
}
```

### Error Logging

With the `logging` feature enabled, all security events are logged appropriately:
- URL validation failures: `WARN` level
- Content security violations: `WARN` level
- Network errors: `ERROR` level

## Usage Examples

### Basic Usage with Default Security

```rust
use url_preview::{PreviewService, PreviewError};

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    // Default configuration includes all security features
    let service = PreviewService::new();
    
    // This will fail due to localhost blocking
    match service.generate_preview("http://localhost:8080").await {
        Err(PreviewError::LocalhostBlocked) => {
            println!("Localhost access blocked as expected");
        }
        _ => unreachable!(),
    }
    
    // This will fail due to private IP blocking
    match service.generate_preview("http://192.168.1.1").await {
        Err(PreviewError::PrivateIpBlocked(_)) => {
            println!("Private IP access blocked as expected");
        }
        _ => unreachable!(),
    }
    
    Ok(())
}
```

### Custom Security Configuration

```rust
use url_preview::{PreviewService, PreviewServiceConfig, FetcherConfig};
use url_preview::{UrlValidationConfig, ContentLimits};
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom URL validation
    let mut url_validation = UrlValidationConfig::default();
    url_validation.allowed_domains.insert("github.com".to_string());
    url_validation.allowed_domains.insert("twitter.com".to_string());
    url_validation.max_redirects = 3;
    
    // Create custom content limits
    let mut content_limits = ContentLimits::default();
    content_limits.max_content_size = 5 * 1024 * 1024; // 5MB
    content_limits.max_download_time = 20; // 20 seconds
    
    // Configure fetcher
    let fetcher_config = FetcherConfig {
        url_validation,
        content_limits,
        ..Default::default()
    };
    
    // Create service with custom config
    let service_config = PreviewServiceConfig {
        fetcher_config: Some(fetcher_config),
        ..Default::default()
    };
    
    let service = PreviewService::with_config(service_config);
    
    // Only github.com and twitter.com URLs will work
    let preview = service.generate_preview("https://github.com/rust-lang/rust").await?;
    println!("Title: {:?}", preview.title);
    
    Ok(())
}
```

### Handling Security Errors

```rust
use url_preview::{PreviewService, PreviewError};

async fn safe_preview(url: &str) -> Result<String, String> {
    let service = PreviewService::new();
    
    match service.generate_preview(url).await {
        Ok(preview) => Ok(preview.title.unwrap_or_else(|| "No title".to_string())),
        Err(e) => match e {
            PreviewError::InvalidUrlScheme(scheme) => {
                Err(format!("Unsafe URL scheme: {}", scheme))
            }
            PreviewError::PrivateIpBlocked(ip) => {
                Err(format!("Access to private IP {} is blocked", ip))
            }
            PreviewError::LocalhostBlocked => {
                Err("Access to localhost is blocked".to_string())
            }
            PreviewError::DomainBlocked(domain) => {
                Err(format!("Domain {} is blocked", domain))
            }
            PreviewError::ContentSizeExceeded { size, limit } => {
                Err(format!("Content too large: {} bytes (limit: {})", size, limit))
            }
            PreviewError::ContentTypeNotAllowed(ct) => {
                Err(format!("Content type {} not allowed", ct))
            }
            _ => Err(format!("Preview failed: {}", e)),
        }
    }
}
```

## Best Practices

### 1. **Always Use HTTPS When Possible**

```rust
let mut config = UrlValidationConfig::default();
config.allowed_schemes.clear();
config.allowed_schemes.insert("https".to_string());
```

### 2. **Implement Domain Whitelisting for Production**

```rust
let mut config = UrlValidationConfig::default();
// Only allow specific trusted domains
config.allowed_domains.insert("trusted-news-site.com".to_string());
config.allowed_domains.insert("official-blog.com".to_string());
```

### 3. **Adjust Limits Based on Use Case**

```rust
// For previewing only lightweight pages
let mut limits = ContentLimits::default();
limits.max_content_size = 1 * 1024 * 1024; // 1MB
limits.max_download_time = 10; // 10 seconds

// For specific content types only
limits.allowed_content_types.clear();
limits.allowed_content_types.insert("text/html".to_string());
```

### 4. **Monitor Security Events**

```rust
use url_preview::setup_logging;

// Enable logging to monitor security events
setup_logging(None);

// Security events will be logged at appropriate levels
```

### 5. **Regular Security Updates**

- Keep the library updated to get latest security patches
- Review and update domain lists periodically
- Monitor for new attack vectors and adjust configuration

## Security Considerations

### What This Library Protects Against

1. **SSRF (Server-Side Request Forgery)**
   - Private IP filtering
   - Localhost blocking
   - Scheme validation

2. **Resource Exhaustion**
   - Content size limits
   - Download time limits
   - Connection pooling
   - Concurrent request limiting

3. **Malicious Redirects**
   - Redirect count limiting
   - Each redirect is validated

4. **Unexpected Content**
   - Content type filtering
   - UTF-8 validation

### What This Library Does NOT Protect Against

1. **JavaScript Execution**
   - The library only parses HTML, it doesn't execute JavaScript
   - Dynamic content loaded by JavaScript won't be captured

2. **Advanced SSRF Techniques**
   - DNS rebinding attacks
   - Time-of-check to time-of-use (TOCTOU) attacks
   - IPv6 address confusion

3. **Content Validation**
   - The library doesn't sanitize extracted content
   - XSS prevention is the responsibility of the consuming application

4. **Rate Limiting Per Domain**
   - The library limits total concurrent requests but not per-domain
   - Implement additional rate limiting if needed

## Conclusion

The `url-preview` library provides comprehensive security features suitable for most URL preview generation use cases. By following the configuration guidelines and best practices outlined in this document, you can ensure safe and secure preview generation in your applications.

For additional security requirements or custom implementations, consider extending the validation logic or implementing additional layers of security in your application.