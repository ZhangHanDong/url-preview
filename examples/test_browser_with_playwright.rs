use url_preview::{PreviewService, PreviewServiceConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing URL Preview with Playwright MCP Server");
    println!("{}", "=".repeat(60));
    
    // Create a basic preview service
    let config = PreviewServiceConfig {
        timeout: Duration::from_secs(30),
        max_concurrent_requests: 5,
        cache_capacity: 100,
        ..Default::default()
    };
    
    let service = PreviewService::new(config)?;
    
    // Test URLs that require JavaScript rendering
    let test_urls = vec![
        ("https://twitter.com/elonmusk", "Twitter profile (requires JS)"),
        ("https://www.reddit.com/r/rust", "Reddit (SPA)"),
        ("https://github.com/rust-lang/rust", "GitHub repository"),
        ("https://example.com", "Simple static page"),
        ("https://react.dev", "React documentation (SPA)"),
    ];
    
    for (url, description) in test_urls {
        println!("\n📍 Testing: {}", description);
        println!("   URL: {}", url);
        println!("{}", "-".repeat(60));
        
        match service.generate_preview(url).await {
            Ok(preview) => {
                println!("✅ Preview generated successfully!");
                println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
                println!("   Description: {}", preview.description.as_deref().unwrap_or("N/A"));
                
                if let Some(image) = &preview.image {
                    println!("   Image: {}", image);
                }
                
                if let Some(author) = &preview.author {
                    println!("   Author: {}", author);
                }
                
                println!("   URL: {}", preview.url);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    }
    
    println!("\n\n🎯 Testing complete!");
    
    Ok(())
}