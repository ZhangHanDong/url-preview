//! Complete test for MCP client functionality
//!
//! Run with:
//! ```
//! cargo run --example test_mcp_complete --features browser
//! ```

#[cfg(feature = "browser")]
use url_preview::{BrowserPreviewService, McpConfig, BrowserUsagePolicy, McpTransport};
use std::sync::Arc;

#[cfg(not(feature = "browser"))]
fn main() {
    eprintln!("This example requires the 'browser' feature to be enabled.");
    eprintln!("Run with: cargo run --example test_mcp_complete --features browser");
}

#[cfg(feature = "browser")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Complete MCP Client Test");
    println!("{}", "=".repeat(60));
    
    // Configure browser service
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@playwright/mcp@latest".to_string(),
        ],
        transport: McpTransport::Stdio,
        browser_timeout: 30,
        max_sessions: 5,
    };
    
    let browser_service = Arc::new(BrowserPreviewService::new(
        mcp_config,
        BrowserUsagePolicy::Always // Force browser usage for all URLs
    ));
    
    println!("▶️  Initializing browser service...");
    browser_service.initialize().await?;
    println!("✅ Browser service initialized successfully!");
    
    // Test cases
    let test_urls = vec![
        ("https://example.com", "Simple static page"),
        ("https://www.rust-lang.org", "Rust language website"),
        ("https://react.dev", "React documentation (SPA)"),
        ("https://github.com/rust-lang/rust", "GitHub repository"),
    ];
    
    for (url, description) in test_urls {
        println!("\n📄 Testing: {} - {}", url, description);
        println!("{}", "-".repeat(60));
        
        match browser_service.generate_preview(url).await {
            Ok(preview) => {
                println!("✅ Preview generated successfully!");
                println!("   Title: {}", preview.title.as_deref().unwrap_or("(none)"));
                println!("   Description: {}", 
                    preview.description.as_deref()
                        .map(|d| if d.len() > 80 { 
                            format!("{}...", &d[..80]) 
                        } else { 
                            d.to_string() 
                        })
                        .unwrap_or_else(|| "(none)".to_string())
                );
                if let Some(img) = &preview.image_url {
                    println!("   Image: {}", img);
                }
                if let Some(site) = &preview.site_name {
                    println!("   Site: {}", site);
                }
            }
            Err(e) => {
                println!("❌ Error generating preview: {}", e);
            }
        }
    }
    
    // Test JavaScript-heavy sites
    println!("\n\n🔬 Testing JavaScript-heavy sites:");
    println!("{}", "=".repeat(60));
    
    let js_sites = vec![
        ("https://twitter.com/rustlang", "Twitter/X profile"),
        ("https://www.instagram.com/rust_programming/", "Instagram profile"),
        ("https://www.linkedin.com/company/rust-programming-language/", "LinkedIn company page"),
    ];
    
    for (url, description) in js_sites {
        println!("\n🌐 Testing: {} - {}", url, description);
        match browser_service.generate_preview(url).await {
            Ok(preview) => {
                println!("✅ Successfully rendered with browser!");
                println!("   Title: {}", preview.title.as_deref().unwrap_or("(none)"));
            }
            Err(e) => {
                println!("⚠️  Error: {}", e);
            }
        }
    }
    
    println!("\n\n🎯 All tests completed!");
    
    Ok(())
}