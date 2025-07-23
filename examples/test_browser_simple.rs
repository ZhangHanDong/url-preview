//! Simple test for browser functionality
//!
//! Run with:
//! ```
//! cargo run --example test_browser_simple --features browser
//! ```

#[cfg(feature = "browser")]
use url_preview::{BrowserPreviewService, McpConfig, BrowserUsagePolicy};
#[cfg(feature = "browser")]
use std::sync::Arc;

#[cfg(not(feature = "browser"))]
fn main() {
    eprintln!("This example requires the 'browser' feature to be enabled.");
    eprintln!("Run with: cargo run --example test_browser_simple --features browser");
}

#[cfg(feature = "browser")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing Browser Functionality");
    println!("{}", "=".repeat(60));
    
    // Configure browser service
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@playwright/mcp@latest".to_string(),
        ],
        transport: url_preview::McpTransport::Stdio,
        browser_timeout: 30,
        max_sessions: 5,
    };
    
    let browser_service = Arc::new(BrowserPreviewService::new(
        mcp_config,
        BrowserUsagePolicy::Auto
    ));
    
    println!("✅ Browser service initialized successfully!");
    
    // Test with a simple static page
    println!("\n📄 Testing static page (example.com)");
    match browser_service.generate_preview("https://example.com").await {
        Ok(preview) => {
            println!("✅ Preview generated:");
            println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
            println!("   URL: {}", preview.url);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test with a JavaScript-heavy site
    println!("\n📄 Testing JavaScript site (react.dev)");
    match browser_service.generate_preview("https://react.dev").await {
        Ok(preview) => {
            println!("✅ Preview generated:");
            println!("   Title: {}", preview.title.as_deref().unwrap_or("N/A"));
            println!("   Description: {}", 
                preview.description.as_deref()
                    .map(|d| if d.len() > 80 { &d[..80] } else { d })
                    .unwrap_or("N/A")
            );
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test Browser Usage Policy
    println!("\n🔧 Testing Browser Usage Policy");
    
    // Auto policy (default)
    println!("\nAuto policy test:");
    match browser_service.should_use_browser("https://twitter.com/elonmusk") {
        true => println!("✅ Would use browser for Twitter"),
        false => println!("❌ Would not use browser for Twitter"),
    }
    
    match browser_service.should_use_browser("https://example.com") {
        true => println!("❌ Would use browser for static site"),
        false => println!("✅ Would not use browser for static site"),
    }
    
    println!("\n🎯 Browser testing complete!");
    
    Ok(())
}