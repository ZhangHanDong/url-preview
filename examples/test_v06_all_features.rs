use url_preview::PreviewService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing URL Preview v0.6 Features");
    println!("{}", "=".repeat(80));
    
    // 创建默认的预览服务
    let service = PreviewService::new();
    
    // 1. 测试基本预览功能
    println!("\n1️⃣ Testing Basic Preview Generation");
    println!("{}", "-".repeat(60));
    
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
            if let Some(site_name) = &preview.site_name {
                println!("   Site: {}", site_name);
            }
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
    
    use futures::future::join_all;
    
    let futures = urls.iter().map(|url| service.generate_preview(url));
    let results = join_all(futures).await;
    
    for (i, (url, result)) in urls.iter().zip(results.iter()).enumerate() {
        match result {
            Ok(preview) => {
                println!("✅ [{}] {}: {}", 
                    i + 1, 
                    url, 
                    preview.title.as_deref().unwrap_or("No title")
                );
            }
            Err(e) => {
                println!("❌ [{}] {}: {}", i + 1, url, e);
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
    
    if second_duration.as_millis() > 0 {
        println!("🚀 Speed improvement: {:.2}x faster", 
            first_duration.as_millis() as f64 / second_duration.as_millis() as f64
        );
    }
    
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
    
    // 6. 测试需要 JavaScript 渲染的网站
    println!("\n\n6️⃣ Testing JavaScript-heavy Sites");
    println!("{}", "-".repeat(60));
    
    let js_sites = vec![
        "https://twitter.com/elonmusk",
        "https://www.reddit.com/r/rust",
        "https://react.dev",
    ];
    
    for site in js_sites {
        match service.generate_preview(site).await {
            Ok(preview) => {
                println!("📄 {}", site);
                println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
                println!("   Description: {}", 
                    preview.description.as_deref()
                        .map(|d| if d.len() > 80 { &d[..80] } else { d })
                        .unwrap_or("N/A")
                );
            }
            Err(e) => {
                println!("❌ {} -> {}", site, e);
            }
        }
    }
    
    println!("\n\n🎯 Testing complete!");
    println!("\n📝 Notes:");
    println!("- Browser features require `--features browser` and playwright MCP server");
    println!("- LLM features require `--features llm` and API key configuration");
    
    Ok(())
}