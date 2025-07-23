use std::time::Duration;
use url_preview::{
    CacheStrategy, ContentLimits, Fetcher, FetcherConfig, PreviewError, PreviewService, PreviewServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== URL Preview Content Limits Example ===\n");

    // Example 1: Default content limits
    println!("1. Testing default content limits:");
    test_default_limits().await?;

    // Example 2: Custom size limits
    println!("\n2. Testing custom size limits:");
    test_size_limits().await?;

    // Example 3: Custom time limits
    println!("\n3. Testing custom time limits:");
    test_time_limits().await?;

    // Example 4: Content type filtering
    println!("\n4. Testing content type filtering:");
    test_content_type_filtering().await?;

    // Example 5: Combined restrictions for lightweight scraping
    println!("\n5. Testing lightweight scraping configuration:");
    test_lightweight_config().await?;

    Ok(())
}

async fn test_default_limits() -> Result<(), Box<dyn std::error::Error>> {
    let service = PreviewService::new();

    println!("  Default limits:");
    println!("    - Max content size: 10MB");
    println!("    - Max download time: 30s");
    println!("    - Allowed content types: text/html, application/xhtml+xml, text/plain, application/json");

    // Test a normal webpage (should succeed)
    match service.generate_preview("https://www.rust-lang.org").await {
        Ok(preview) => {
            println!("    ✓ Normal webpage loaded successfully");
            if let Some(title) = preview.title {
                println!("      Title: {}", title);
            }
        }
        Err(e) => println!("    ✗ Failed to load webpage: {}", e),
    }

    Ok(())
}

async fn test_size_limits() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with small size limit
    let mut content_limits = ContentLimits::default();
    content_limits.max_content_size = 1024 * 100; // 100KB limit

    let fetcher_config = FetcherConfig {
        content_limits,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    println!("  Testing with 100KB size limit:");

    // Try to fetch a page that might be larger than 100KB
    match service
        .generate_preview("https://en.wikipedia.org/wiki/Rust_(programming_language)")
        .await
    {
        Ok(_) => println!("    ℹ Page loaded (was under 100KB)"),
        Err(PreviewError::ContentSizeExceeded { size, limit }) => {
            println!(
                "    ✓ Large page blocked: {} bytes > {} bytes limit",
                size, limit
            );
        }
        Err(e) => println!("    ℹ Different error occurred: {}", e),
    }

    // Try a smaller page
    match service.generate_preview("https://example.com").await {
        Ok(_) => println!("    ✓ Small page loaded successfully"),
        Err(e) => println!("    ✗ Small page should load: {}", e),
    }

    Ok(())
}

async fn test_time_limits() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration with short time limit
    let mut content_limits = ContentLimits::default();
    content_limits.max_download_time = 2; // 2 second limit

    let fetcher_config = FetcherConfig {
        content_limits,
        timeout: Duration::from_secs(3), // Overall timeout slightly higher
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    println!("  Testing with 2 second download time limit:");

    // Test with a fast-loading site
    match service.generate_preview("https://example.com").await {
        Ok(_) => println!("    ✓ Fast site loaded within time limit"),
        Err(PreviewError::DownloadTimeExceeded { elapsed, limit }) => {
            println!("    ✗ Fast site timed out: {}s > {}s", elapsed, limit);
        }
        Err(e) => println!("    ℹ Different error: {}", e),
    }

    // Note: Testing actual timeout is difficult without a slow server
    println!("    ℹ Timeout testing requires a slow server to demonstrate");

    Ok(())
}

async fn test_content_type_filtering() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration that only allows HTML
    let mut content_limits = ContentLimits::default();
    content_limits.allowed_content_types.clear();
    content_limits
        .allowed_content_types
        .insert("text/html".to_string());

    let fetcher_config = FetcherConfig {
        content_limits,
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    println!("  Testing with HTML-only content type filter:");

    // Test HTML page (should succeed)
    match service.generate_preview("https://example.com").await {
        Ok(_) => println!("    ✓ HTML page allowed"),
        Err(e) => println!("    ✗ HTML page should be allowed: {}", e),
    }

    // Test JSON API (should fail if server returns application/json)
    match service.generate_preview("https://api.github.com").await {
        Err(PreviewError::ContentTypeNotAllowed(ct)) => {
            println!("    ✓ Non-HTML content blocked: {}", ct);
        }
        Ok(_) => println!("    ℹ API returned HTML or no content-type header"),
        Err(e) => println!("    ℹ Different error: {}", e),
    }

    Ok(())
}

async fn test_lightweight_config() -> Result<(), Box<dyn std::error::Error>> {
    // Create a lightweight configuration for fast, safe scraping
    let mut content_limits = ContentLimits::default();
    content_limits.max_content_size = 1024 * 1024; // 1MB
    content_limits.max_download_time = 5; // 5 seconds
    content_limits.allowed_content_types.clear();
    content_limits
        .allowed_content_types
        .insert("text/html".to_string());
    content_limits
        .allowed_content_types
        .insert("application/xhtml+xml".to_string());

    let fetcher_config = FetcherConfig {
        content_limits,
        timeout: Duration::from_secs(5),
        user_agent: "LightweightBot/1.0".to_string(),
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(100) // Smaller cache
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_max_concurrent_requests(10) // Lower concurrency
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    println!("  Lightweight configuration:");
    println!("    - Max size: 1MB");
    println!("    - Max time: 5s");
    println!("    - HTML only");
    println!("    - Max 10 concurrent requests");
    println!("    - Cache capacity: 100");

    // Test multiple URLs
    let urls = vec![
        "https://example.com",
        "https://www.rust-lang.org",
        "https://doc.rust-lang.org/book/",
    ];

    for url in urls {
        match service.generate_preview(url).await {
            Ok(preview) => {
                println!(
                    "    ✓ {}: {}",
                    url,
                    preview.title.unwrap_or_else(|| "No title".to_string())
                );
            }
            Err(e) => {
                println!("    ✗ {}: {}", url, e);
            }
        }
    }

    Ok(())
}
