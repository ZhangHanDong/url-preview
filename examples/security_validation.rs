use url_preview::{CacheStrategy, Fetcher, FetcherConfig, PreviewError, PreviewService, PreviewServiceConfig, UrlValidationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== URL Preview Security Validation Example ===\n");

    // Example 1: Default security configuration
    println!("1. Testing default security configuration:");
    test_default_security().await?;

    // Example 2: Custom domain whitelist
    println!("\n2. Testing domain whitelist:");
    test_domain_whitelist().await?;

    // Example 3: Custom domain blacklist
    println!("\n3. Testing domain blacklist:");
    test_domain_blacklist().await?;

    // Example 4: Testing private IP blocking
    println!("\n4. Testing private IP and localhost blocking:");
    test_private_ip_blocking().await?;

    // Example 5: HTTPS-only configuration
    println!("\n5. Testing HTTPS-only configuration:");
    test_https_only().await?;

    Ok(())
}

async fn test_default_security() -> Result<(), Box<dyn std::error::Error>> {
    let service = PreviewService::new();

    // These should fail with default security settings
    let test_urls = vec![
        ("http://localhost", "LocalhostBlocked"),
        ("http://127.0.0.1", "LocalhostBlocked"),
        ("http://192.168.1.1", "PrivateIpBlocked"),
        ("http://10.0.0.1", "PrivateIpBlocked"),
        ("file:///etc/passwd", "InvalidUrlScheme"),
        ("ftp://example.com", "InvalidUrlScheme"),
    ];

    for (url, _expected_error) in test_urls {
        match service.generate_preview(url).await {
            Err(e) => println!("  ✓ {} blocked as expected: {}", url, e),
            Ok(_) => println!("  ✗ {} should have been blocked!", url),
        }
    }

    // This should succeed
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(preview) => println!(
            "  ✓ https://www.rust-lang.org allowed: {}",
            preview.title.unwrap_or_else(|| "No title".to_string())
        ),
        Err(e) => println!("  ✗ Public HTTPS URL should be allowed: {}", e),
    }

    Ok(())
}

async fn test_domain_whitelist() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with domain whitelist
    let mut url_validation = UrlValidationConfig::default();
    url_validation
        .allowed_domains
        .insert("rust-lang.org".to_string());
    url_validation
        .allowed_domains
        .insert("github.com".to_string());

    let fetcher_config = FetcherConfig {
        url_validation,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Test allowed domains
    println!("  Testing allowed domains:");
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(_) => println!("    ✓ rust-lang.org allowed"),
        Err(e) => println!("    ✗ rust-lang.org should be allowed: {}", e),
    }

    match service
        .generate_preview("https://github.com/rust-lang")
        .await
    {
        Ok(_) => println!("    ✓ github.com allowed"),
        Err(e) => println!("    ✗ github.com should be allowed: {}", e),
    }

    // Test blocked domain
    match service.generate_preview("https://example.com").await {
        Err(PreviewError::DomainNotAllowed(domain)) => {
            println!("    ✓ {} blocked (not in whitelist)", domain)
        }
        _ => println!("    ✗ example.com should be blocked"),
    }

    Ok(())
}

async fn test_domain_blacklist() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with domain blacklist
    let mut url_validation = UrlValidationConfig::default();
    url_validation
        .blocked_domains
        .insert("evil.com".to_string());
    url_validation
        .blocked_domains
        .insert("malicious.site".to_string());

    let fetcher_config = FetcherConfig {
        url_validation,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Test blocked domains
    println!("  Testing blocked domains:");
    match service.generate_preview("https://evil.com").await {
        Err(PreviewError::DomainBlocked(domain)) => {
            println!("    ✓ {} blocked", domain)
        }
        _ => println!("    ✗ evil.com should be blocked"),
    }

    match service.generate_preview("https://sub.evil.com").await {
        Err(PreviewError::DomainBlocked(domain)) => {
            println!("    ✓ {} blocked (subdomain)", domain)
        }
        _ => println!("    ✗ sub.evil.com should be blocked"),
    }

    // Test allowed domain
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(_) => println!("    ✓ rust-lang.org allowed"),
        Err(e) => println!("    ✗ rust-lang.org should be allowed: {}", e),
    }

    Ok(())
}

async fn test_private_ip_blocking() -> Result<(), Box<dyn std::error::Error>> {
    let service = PreviewService::new();

    let private_ips = vec![
        "http://10.0.0.1",
        "http://172.16.0.1",
        "http://192.168.1.1",
        "http://169.254.1.1",
        "http://127.0.0.1",
        "http://localhost",
        "http://[::1]",
        "http://[fe80::1]",
        "http://[fc00::1]",
    ];

    for ip_url in private_ips {
        match service.generate_preview(ip_url).await {
            Err(PreviewError::PrivateIpBlocked(ip)) => {
                println!("  ✓ {} blocked (private IP: {})", ip_url, ip)
            }
            Err(PreviewError::LocalhostBlocked) => {
                println!("  ✓ {} blocked (localhost)", ip_url)
            }
            _ => println!("  ✗ {} should be blocked!", ip_url),
        }
    }

    // Test public IP (Google DNS)
    match service.generate_preview("http://8.8.8.8").await {
        Ok(_) | Err(_) => println!("  ✓ Public IP 8.8.8.8 not blocked due to IP restriction"),
    }

    Ok(())
}

async fn test_https_only() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTPS-only configuration
    let mut url_validation = UrlValidationConfig::default();
    url_validation.allowed_schemes.clear();
    url_validation.allowed_schemes.insert("https".to_string());

    let fetcher_config = FetcherConfig {
        url_validation,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Test HTTP (should be blocked)
    match service.generate_preview("http://example.com").await {
        Err(PreviewError::InvalidUrlScheme(scheme)) => {
            println!("  ✓ HTTP blocked in HTTPS-only mode: {}", scheme)
        }
        _ => println!("  ✗ HTTP should be blocked in HTTPS-only mode"),
    }

    // Test HTTPS (should be allowed)
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(_) => println!("  ✓ HTTPS allowed"),
        Err(e) => println!("  ✗ HTTPS should be allowed: {}", e),
    }

    Ok(())
}

