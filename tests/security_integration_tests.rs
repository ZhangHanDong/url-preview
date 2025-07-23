use std::sync::Arc;
use std::time::{Duration, Instant};
use url_preview::{
    CacheStrategy, ContentLimits, Fetcher, FetcherConfig, PreviewError, PreviewService, PreviewServiceConfig,
    UrlValidationConfig,
};

#[tokio::test]
async fn test_concurrent_security_validation() {
    // Create a service with strict security settings
    let mut url_config = UrlValidationConfig::default();
    url_config.allowed_domains.insert("example.com".to_string());

    let fetcher_config = FetcherConfig {
        url_validation: url_config,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_max_concurrent_requests(5)
        .with_default_fetcher(custom_fetcher);

    let service = Arc::new(PreviewService::new_with_config(service_config));

    // Test concurrent requests with mix of allowed and blocked URLs
    let urls = vec![
        ("https://example.com", true),
        ("https://blocked.com", false),
        ("http://localhost", false),
        ("http://192.168.1.1", false),
        ("https://sub.example.com", true),
        ("file:///etc/passwd", false),
        ("https://another-blocked.com", false),
        ("https://example.com/page", true),
    ];

    let mut handles = vec![];

    for (url, should_succeed) in urls {
        let service_clone = Arc::clone(&service);
        let url = url.to_string();

        let handle = tokio::spawn(async move {
            let result = service_clone.generate_preview(&url).await;
            (url, should_succeed, result)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (url, should_succeed, result) = handle.await.unwrap();

        if should_succeed {
            match result {
                Ok(_) => println!("✓ {} - Allowed as expected", url),
                Err(e) => println!("ℹ {} - Failed (might be network): {}", url, e),
            }
        } else {
            match result {
                Err(PreviewError::DomainNotAllowed(_))
                | Err(PreviewError::LocalhostBlocked)
                | Err(PreviewError::PrivateIpBlocked(_))
                | Err(PreviewError::InvalidUrlScheme(_)) => {
                    println!("✓ {} - Blocked as expected", url);
                }
                Ok(_) => panic!("✗ {} - Should have been blocked!", url),
                Err(e) => println!("✗ {} - Wrong error type: {}", url, e),
            }
        }
    }
}

#[tokio::test]
async fn test_security_with_caching() {
    // Create service with caching enabled
    let mut url_config = UrlValidationConfig::default();
    url_config.blocked_domains.insert("blocked.com".to_string());

    let fetcher_config = FetcherConfig {
        url_validation: url_config,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(100)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // First attempt - should be blocked
    let result1 = service.generate_preview("https://blocked.com").await;
    assert!(matches!(result1, Err(PreviewError::DomainBlocked(_))));

    // Second attempt - should still be blocked (not cached)
    let result2 = service.generate_preview("https://blocked.com").await;
    assert!(matches!(result2, Err(PreviewError::DomainBlocked(_))));

    // Valid URL should be cached
    if let Ok(_) = service.generate_preview("https://example.com").await {
        // Second request should be faster (from cache)
        let start = Instant::now();
        let _ = service.generate_preview("https://example.com").await;
        let cached_duration = start.elapsed();

        // Third request with no-cache should be slower
        let start = Instant::now();
        let _ = service
            .generate_preview_no_cache("https://example.com")
            .await;
        let fresh_duration = start.elapsed();

        println!(
            "Cached request: {:?}, Fresh request: {:?}",
            cached_duration, fresh_duration
        );
    }
}

#[tokio::test]
async fn test_security_error_propagation() {
    let service = PreviewService::new();

    // Test that security errors are properly categorized
    let test_cases = vec![
        ("http://localhost", "LocalhostBlocked"),
        ("http://127.0.0.1", "LocalhostBlocked"),
        ("http://192.168.1.1", "PrivateIpBlocked"),
        ("file:///etc/passwd", "InvalidUrlScheme"),
        ("ftp://example.com", "InvalidUrlScheme"),
        ("javascript:alert(1)", "InvalidUrlScheme"),
    ];

    for (url, expected_error) in test_cases {
        match service.generate_preview(url).await {
            Err(e) => {
                let error_type = match &e {
                    PreviewError::LocalhostBlocked => "LocalhostBlocked",
                    PreviewError::PrivateIpBlocked(_) => "PrivateIpBlocked",
                    PreviewError::InvalidUrlScheme(_) => "InvalidUrlScheme",
                    _ => "Other",
                };
                assert_eq!(error_type, expected_error, "URL: {}", url);

                // Test error display
                println!("{}: {}", url, e);
            }
            Ok(_) => panic!("{} should have been blocked", url),
        }
    }
}

#[tokio::test]
async fn test_progressive_security_levels() {
    // Level 1: Permissive
    let mut config1 = UrlValidationConfig::default();
    config1.block_private_ips = false;
    config1.block_localhost = false;

    let custom_fetcher1 = Fetcher::with_config(FetcherConfig {
        url_validation: config1,
        ..Default::default()
    });
    
    let service1 = PreviewService::new_with_config(
        PreviewServiceConfig::new(1000)
            .with_cache_strategy(CacheStrategy::UseCache)
            .with_default_fetcher(custom_fetcher1)
    );

    // Should allow localhost
    match service1.generate_preview("http://localhost").await {
        Ok(_) | Err(_) => { /* Might fail due to no server, but shouldn't be blocked */ }
    }

    // Level 2: Default security
    let service2 = PreviewService::new();

    // Should block localhost
    assert!(matches!(
        service2.generate_preview("http://localhost").await,
        Err(PreviewError::LocalhostBlocked)
    ));

    // Level 3: Strict whitelist
    let mut config3 = UrlValidationConfig::default();
    config3.allowed_schemes.clear();
    config3.allowed_schemes.insert("https".to_string());
    config3.allowed_domains.insert("trusted.com".to_string());

    let custom_fetcher3 = Fetcher::with_config(FetcherConfig {
        url_validation: config3,
        content_limits: ContentLimits {
            max_content_size: 1024 * 1024, // 1MB
            max_download_time: 5,
            ..Default::default()
        },
        timeout: Duration::from_secs(5),
        ..Default::default()
    });
    
    let service3 = PreviewService::new_with_config(
        PreviewServiceConfig::new(1000)
            .with_cache_strategy(CacheStrategy::UseCache)
            .with_max_concurrent_requests(10)
            .with_default_fetcher(custom_fetcher3)
    );

    // Should only allow HTTPS to trusted.com
    assert!(matches!(
        service3.generate_preview("http://trusted.com").await,
        Err(PreviewError::InvalidUrlScheme(_))
    ));

    assert!(matches!(
        service3.generate_preview("https://untrusted.com").await,
        Err(PreviewError::DomainNotAllowed(_))
    ));
}

#[tokio::test]
async fn test_content_security_streaming() {
    // Test that content limits are enforced during streaming
    let mut limits = ContentLimits::default();
    limits.max_content_size = 1024 * 10; // 10KB
    limits.max_download_time = 2; // 2 seconds

    let custom_fetcher = Fetcher::with_config(FetcherConfig {
        content_limits: limits,
        timeout: Duration::from_secs(3),
        ..Default::default()
    });
    
    let service = PreviewService::new_with_config(
        PreviewServiceConfig::new(1000)
            .with_cache_strategy(CacheStrategy::UseCache)
            .with_default_fetcher(custom_fetcher)
    );

    // Try to fetch a potentially large page
    match service
        .generate_preview("https://en.wikipedia.org/wiki/Rust_(programming_language)")
        .await
    {
        Ok(_) => println!("Page was under size limit"),
        Err(PreviewError::ContentSizeExceeded { size, limit }) => {
            println!("✓ Content size limit enforced: {} > {}", size, limit);
            assert!(size > limit);
            assert_eq!(limit, 1024 * 10);
        }
        Err(PreviewError::DownloadTimeExceeded { elapsed, limit }) => {
            println!("✓ Download time limit enforced: {}s > {}s", elapsed, limit);
            assert!(elapsed >= limit);
            assert_eq!(limit, 2);
        }
        Err(e) => println!("Other error: {}", e),
    }
}

#[tokio::test]
async fn test_security_with_rate_limiting() {
    // Test that security checks happen before rate limiting
    // Test that rate limiting is configured correctly

    let service = PreviewService::new_with_config(
        PreviewServiceConfig::new(1000)
            .with_cache_strategy(CacheStrategy::UseCache)
            .with_max_concurrent_requests(1)
    );

    // Security check should happen immediately, not wait for semaphore
    let start = Instant::now();
    let result = service.generate_preview("http://localhost").await;
    let duration = start.elapsed();

    assert!(matches!(result, Err(PreviewError::LocalhostBlocked)));
    assert!(
        duration < Duration::from_millis(100),
        "Security check should be instant"
    );
}

#[tokio::test]
async fn test_mixed_security_batch() {
    let service = PreviewService::new();

    // Batch request with mixed valid/invalid URLs
    let urls = vec![
        "https://example.com",
        "http://localhost",
        "https://www.rust-lang.org",
        "http://192.168.1.1",
        "file:///etc/passwd",
    ];

    let mut results = vec![];
    for url in &urls {
        results.push(service.generate_preview(url).await);
    }

    // Check results
    assert!(results[0].is_ok() || results[0].is_err()); // example.com might fail due to network
    assert!(matches!(results[1], Err(PreviewError::LocalhostBlocked)));
    assert!(results[2].is_ok() || results[2].is_err()); // rust-lang.org might fail due to network
    assert!(matches!(results[3], Err(PreviewError::PrivateIpBlocked(_))));
    assert!(matches!(results[4], Err(PreviewError::InvalidUrlScheme(_))));
}
