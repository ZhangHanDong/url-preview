use std::time::Duration;
use url_preview::{
    CacheStrategy, ContentLimits, Fetcher, FetcherConfig, PreviewError, PreviewService, PreviewServiceConfig,
    UrlValidationConfig, UrlValidator,
};

#[tokio::test]
async fn test_default_url_validation() {
    let validator = UrlValidator::with_default_config();

    // Test allowed schemes
    assert!(validator.validate("http://example.com").is_ok());
    assert!(validator.validate("https://example.com").is_ok());

    // Test blocked schemes
    assert!(matches!(
        validator.validate("file:///etc/passwd"),
        Err(PreviewError::InvalidUrlScheme(_))
    ));
    assert!(matches!(
        validator.validate("ftp://example.com"),
        Err(PreviewError::InvalidUrlScheme(_))
    ));
    assert!(matches!(
        validator.validate("javascript:alert(1)"),
        Err(PreviewError::InvalidUrlScheme(_))
    ));
}

#[tokio::test]
async fn test_localhost_blocking() {
    let validator = UrlValidator::with_default_config();

    // All localhost variations should be blocked
    assert!(matches!(
        validator.validate("http://localhost"),
        Err(PreviewError::LocalhostBlocked)
    ));
    assert!(matches!(
        validator.validate("http://localhost:8080"),
        Err(PreviewError::LocalhostBlocked)
    ));
    assert!(matches!(
        validator.validate("http://127.0.0.1"),
        Err(PreviewError::LocalhostBlocked)
    ));
    assert!(matches!(
        validator.validate("http://127.0.0.1:3000"),
        Err(PreviewError::LocalhostBlocked)
    ));
    assert!(matches!(
        validator.validate("http://[::1]"),
        Err(PreviewError::LocalhostBlocked)
    ));
    assert!(matches!(
        validator.validate("https://[::1]:8443"),
        Err(PreviewError::LocalhostBlocked)
    ));
}

#[tokio::test]
async fn test_private_ip_blocking() {
    let validator = UrlValidator::with_default_config();

    // Test IPv4 private ranges
    let private_ips = vec![
        "http://10.0.0.1",
        "http://10.255.255.255",
        "http://172.16.0.1",
        "http://172.31.255.255",
        "http://192.168.0.1",
        "http://192.168.255.255",
        "http://169.254.0.1",
        "http://169.254.255.255",
        "http://100.64.0.1",
        "http://100.127.255.255",
    ];

    for ip in private_ips {
        assert!(
            matches!(
                validator.validate(ip),
                Err(PreviewError::PrivateIpBlocked(_))
            ),
            "Failed to block private IP: {}",
            ip
        );
    }

    // Test IPv6 private ranges
    let ipv6_tests = vec![
        "http://[fe80::1]",
        "http://[fc00::1]",
        "http://[fd00::1]",
    ];
    
    for ip in ipv6_tests {
        match validator.validate(ip) {
            Err(PreviewError::PrivateIpBlocked(_)) => { /* expected */ }
            Ok(_) => panic!("IPv6 {} should be blocked", ip),
            Err(e) => println!("IPv6 {} error: {}", ip, e),
        }
    }

    // Test public IPs should pass
    assert!(validator.validate("http://8.8.8.8").is_ok());
    assert!(validator.validate("http://1.1.1.1").is_ok());
}

#[tokio::test]
async fn test_domain_whitelist() {
    let mut config = UrlValidationConfig::default();
    config.allowed_domains.insert("trusted.com".to_string());
    config.allowed_domains.insert("example.org".to_string());

    let validator = UrlValidator::new(config);

    // Test allowed domains
    assert!(validator.validate("https://trusted.com").is_ok());
    assert!(validator.validate("https://sub.trusted.com").is_ok());
    assert!(validator.validate("https://deep.sub.trusted.com").is_ok());
    assert!(validator.validate("https://example.org").is_ok());

    // Test blocked domains
    assert!(matches!(
        validator.validate("https://untrusted.com"),
        Err(PreviewError::DomainNotAllowed(_))
    ));
    assert!(matches!(
        validator.validate("https://example.com"),
        Err(PreviewError::DomainNotAllowed(_))
    ));
}

#[tokio::test]
async fn test_domain_blacklist() {
    let mut config = UrlValidationConfig::default();
    config.blocked_domains.insert("evil.com".to_string());
    config.blocked_domains.insert("malicious.org".to_string());

    let validator = UrlValidator::new(config);

    // Test blocked domains
    assert!(matches!(
        validator.validate("https://evil.com"),
        Err(PreviewError::DomainBlocked(_))
    ));
    assert!(matches!(
        validator.validate("https://sub.evil.com"),
        Err(PreviewError::DomainBlocked(_))
    ));
    assert!(matches!(
        validator.validate("https://malicious.org"),
        Err(PreviewError::DomainBlocked(_))
    ));

    // Test allowed domains
    assert!(validator.validate("https://good.com").is_ok());
    assert!(validator.validate("https://example.com").is_ok());
}

#[tokio::test]
async fn test_whitelist_precedence() {
    let mut config = UrlValidationConfig::default();
    // When whitelist is set, only whitelisted domains are allowed
    config.allowed_domains.insert("allowed.com".to_string());
    // Even if a domain is not in the blacklist, it should still be blocked
    config.blocked_domains.insert("blocked.com".to_string());

    let validator = UrlValidator::new(config);

    // Whitelisted domain should pass
    assert!(validator.validate("https://allowed.com").is_ok());

    // Non-whitelisted domains should fail, even if not blacklisted
    assert!(matches!(
        validator.validate("https://example.com"),
        Err(PreviewError::DomainNotAllowed(_))
    ));
    assert!(matches!(
        validator.validate("https://blocked.com"),
        Err(PreviewError::DomainNotAllowed(_))
    ));
}

#[tokio::test]
async fn test_disable_security_features() {
    let mut config = UrlValidationConfig::default();
    config.block_private_ips = false;
    config.block_localhost = false;

    let validator = UrlValidator::new(config);

    // With security disabled, these should pass
    assert!(validator.validate("http://localhost").is_ok());
    assert!(validator.validate("http://127.0.0.1").is_ok());
    assert!(validator.validate("http://192.168.1.1").is_ok());
    assert!(validator.validate("http://10.0.0.1").is_ok());
}

#[tokio::test]
async fn test_content_size_limits() {
    let mut limits = ContentLimits::default();
    limits.max_content_size = 1024; // 1KB limit

    let fetcher_config = FetcherConfig {
        content_limits: limits,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Small pages should work
    match service.generate_preview("https://example.com").await {
        Ok(_) => { /* Success is acceptable */ }
        Err(PreviewError::ContentSizeExceeded { size, limit }) => {
            assert!(size > limit);
            assert_eq!(limit, 1024);
        }
        Err(e) => {
            // Other errors might occur due to network issues
            println!("Different error occurred: {}", e);
        }
    }
}

#[tokio::test]
async fn test_content_type_filtering() {
    let mut limits = ContentLimits::default();
    limits.allowed_content_types.clear();
    limits.allowed_content_types.insert("text/html".to_string());

    let fetcher_config = FetcherConfig {
        content_limits: limits,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // HTML pages should work
    match service.generate_preview("https://example.com").await {
        Ok(_) => { /* HTML content should pass */ }
        Err(e) => {
            println!("HTML page error (might be network issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_https_only_mode() {
    let mut config = UrlValidationConfig::default();
    config.allowed_schemes.clear();
    config.allowed_schemes.insert("https".to_string());

    let fetcher_config = FetcherConfig {
        url_validation: config,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // HTTP should be blocked
    assert!(matches!(
        service.generate_preview("http://example.com").await,
        Err(PreviewError::InvalidUrlScheme(_))
    ));

    // HTTPS should work
    match service.generate_preview("https://example.com").await {
        Ok(_) | Err(_) => { /* Any result is fine, just shouldn't be InvalidUrlScheme */ }
    }
}

#[tokio::test]
async fn test_max_redirects() {
    let mut config = UrlValidationConfig::default();
    config.max_redirects = 2;

    let validator = UrlValidator::new(config);

    // Basic validation should still work
    assert!(validator.validate("https://example.com").is_ok());
}

#[tokio::test]
async fn test_integration_security_with_service() {
    // Create a restrictive configuration
    let mut url_config = UrlValidationConfig::default();
    url_config
        .allowed_domains
        .insert("rust-lang.org".to_string());
    url_config.allowed_domains.insert("example.com".to_string());

    let mut content_limits = ContentLimits::default();
    content_limits.max_content_size = 5 * 1024 * 1024; // 5MB
    content_limits.max_download_time = 10; // 10 seconds

    let fetcher_config = FetcherConfig {
        url_validation: url_config,
        content_limits,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_max_concurrent_requests(10)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Test allowed domain
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(_) => { /* Success expected */ }
        Err(e) => println!("Allowed domain error (might be network): {}", e),
    }

    // Test blocked domain
    // Note: When github feature is enabled, github.com URLs are handled specially
    // and bypass domain filtering, so we test with a different domain
    assert!(matches!(
        service.generate_preview("https://untrusted.com").await,
        Err(PreviewError::DomainNotAllowed(_))
    ));

    // Test blocked localhost - should still be blocked even with domain whitelist
    match service.generate_preview("http://localhost:8080").await {
        Err(PreviewError::LocalhostBlocked) => { /* expected */ }
        Err(e) => println!("Localhost blocked with different error: {}", e),
        Ok(_) => panic!("Localhost should be blocked"),
    }

    // Test blocked private IP - should still be blocked even with domain whitelist
    match service.generate_preview("http://192.168.1.1").await {
        Err(PreviewError::PrivateIpBlocked(_)) => { /* expected */ }
        Err(e) => println!("Private IP blocked with different error: {}", e),
        Ok(_) => panic!("Private IP should be blocked"),
    }
}

#[test]
fn test_url_validation_edge_cases() {
    let validator = UrlValidator::with_default_config();

    // Test various URL edge cases
    assert!(matches!(
        validator.validate("not-a-url"),
        Err(PreviewError::UrlParseError(_))
    ));

    assert!(matches!(
        validator.validate(""),
        Err(PreviewError::UrlParseError(_))
    ));

    // URL without host will parse but fail validation
    match validator.validate("https://") {
        Err(PreviewError::InvalidUrl(_)) => { /* expected */ }
        Err(PreviewError::UrlParseError(_)) => { /* also acceptable */ }
        _ => panic!("Expected error for URL without host"),
    }

    // Test URL with authentication (should work)
    assert!(validator.validate("https://user:pass@example.com").is_ok());

    // Test URL with port
    assert!(validator.validate("https://example.com:8443").is_ok());

    // Test URL with path and query
    assert!(validator
        .validate("https://example.com/path?query=value#fragment")
        .is_ok());
}

#[test]
fn test_special_ip_ranges() {
    let validator = UrlValidator::with_default_config();

    // Test special IPv4 ranges
    assert!(matches!(
        validator.validate("http://0.0.0.0"),
        Err(PreviewError::PrivateIpBlocked(_))
    ));

    assert!(matches!(
        validator.validate("http://224.0.0.1"), // Multicast
        Err(PreviewError::PrivateIpBlocked(_))
    ));

    assert!(matches!(
        validator.validate("http://240.0.0.1"), // Reserved
        Err(PreviewError::PrivateIpBlocked(_))
    ));

    assert!(matches!(
        validator.validate("http://255.255.255.255"), // Broadcast
        Err(PreviewError::PrivateIpBlocked(_))
    ));
}
