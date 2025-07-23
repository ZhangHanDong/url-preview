use url_preview::{PreviewService, PreviewServiceConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing URL Preview v0.6 Features");
    println!("{}", "=".repeat(80));
    
    // 1. 测试基本预览功能
    println!("\n1️⃣ Testing Basic Preview Generation");
    println!("{}", "-".repeat(60));
    
    let config = PreviewServiceConfig {
        timeout: Duration::from_secs(30),
        max_concurrent_requests: 5,
        cache_capacity: 100,
        ..Default::default()
    };
    
    let service = PreviewService::new(config)?;
    
    // 测试静态页面
    match service.generate_preview("https://example.com").await {
        Ok(preview) => {
            println!("✅ Static page preview:");
            println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
            println!("   Description: {}", preview.description.as_deref().unwrap_or("N/A"));
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // 测试 GitHub
    match service.generate_preview("https://github.com/rust-lang/rust").await {
        Ok(preview) => {
            println!("\n✅ GitHub repository preview:");
            println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
            println!("   Description: {}", preview.description.as_deref().unwrap_or("N/A"));
            println!("   Author: {}", preview.author.as_deref().unwrap_or("N/A"));
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // 2. 测试并发处理
    println!("\n\n2️⃣ Testing Concurrent Preview Generation");
    println!("{}", "-".repeat(60));
    
    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com/tokio-rs/tokio",
        "https://example.com",
        "https://httpbin.org/html",
    ];
    
    let results = service.generate_previews_batch(&urls).await;
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(preview) => {
                println!("✅ [{}] {}: {}", 
                    i + 1, 
                    urls[i], 
                    preview.title.as_deref().unwrap_or("No title")
                );
            }
            Err(e) => {
                println!("❌ [{}] {}: {}", i + 1, urls[i], e);
            }
        }
    }
    
    // 3. 测试缓存功能
    println!("\n\n3️⃣ Testing Cache Functionality");
    println!("{}", "-".repeat(60));
    
    let test_url = "https://www.rust-lang.org";
    
    // 第一次请求（未缓存）
    let start = std::time::Instant::now();
    let _ = service.generate_preview(test_url).await?;
    let first_duration = start.elapsed();
    println!("⏱️  First request (uncached): {:?}", first_duration);
    
    // 第二次请求（已缓存）
    let start = std::time::Instant::now();
    let _ = service.generate_preview(test_url).await?;
    let second_duration = start.elapsed();
    println!("⏱️  Second request (cached): {:?}", second_duration);
    println!("🚀 Speed improvement: {:.2}x faster", 
        first_duration.as_millis() as f64 / second_duration.as_millis() as f64
    );
    
    // 4. 测试错误处理（v0.4 新功能）
    println!("\n\n4️⃣ Testing Error Handling (v0.4 features)");
    println!("{}", "-".repeat(60));
    
    let error_urls = vec![
        ("https://httpbin.org/status/404", "404 Not Found"),
        ("https://httpbin.org/status/500", "500 Server Error"),
        ("https://invalid-domain-that-does-not-exist.com", "DNS Error"),
        ("https://github.com/non-existent-repo/does-not-exist", "GitHub 404"),
    ];
    
    for (url, expected) in error_urls {
        match service.generate_preview(url).await {
            Ok(_) => println!("🤔 Unexpected success for: {}", url),
            Err(e) => println!("✅ {} -> {}", expected, e),
        }
    }
    
    // 5. 测试安全功能（v0.5 新功能）
    println!("\n\n5️⃣ Testing Security Features (v0.5)");
    println!("{}", "-".repeat(60));
    
    let security_test_urls = vec![
        ("http://localhost:8080", "Localhost blocked"),
        ("http://127.0.0.1", "Localhost IP blocked"),
        ("http://192.168.1.1", "Private IP blocked"),
        ("http://10.0.0.1", "Private IP blocked"),
        ("file:///etc/passwd", "Invalid scheme blocked"),
    ];
    
    for (url, expected) in security_test_urls {
        match service.generate_preview(url).await {
            Ok(_) => println!("⚠️  Security bypass for: {}", url),
            Err(e) => println!("✅ {} -> {}", expected, e),
        }
    }
    
    println!("\n\n🎯 Basic testing complete!");
    println!("\nNote: Browser and LLM features require additional setup:");
    println!("- Browser: Requires playwright MCP server running");
    println!("- LLM: Requires API keys configuration");
    
    Ok(())
}