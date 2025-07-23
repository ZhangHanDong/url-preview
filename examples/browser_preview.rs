//! Example of using browser-based preview generation with playwright-mcp
//!
//! This example demonstrates how to use the browser feature to generate
//! previews for JavaScript-heavy websites and SPAs.
//!
//! Prerequisites:
//! 1. Install Node.js and npm
//! 2. The MCP server will be automatically installed via npx
//!
//! Run with:
//! ```
//! cargo run --example browser_preview --features browser
//! ```

#[cfg(feature = "browser")]
use url_preview::{
    BrowserUsagePolicy, McpConfig, McpTransport, PreviewService, PreviewServiceConfig,
};

#[cfg(not(feature = "browser"))]
fn main() {
    eprintln!("This example requires the 'browser' feature to be enabled.");
    eprintln!("Run with: cargo run --example browser_preview --features browser");
    std::process::exit(1);
}

#[cfg(feature = "browser")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Browser-based URL Preview Example\n");
    
    // Configure MCP for browser automation
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@playwright/mcp@latest".to_string(),
        ],
        transport: McpTransport::Stdio,
        browser_timeout: 30,
        max_sessions: 3,
    };
    
    // Create service with browser support
    let config = PreviewServiceConfig::new(1000)
        .with_mcp_config(mcp_config)
        .with_browser_usage_policy(BrowserUsagePolicy::Auto);
    
    let service = PreviewService::new_with_config(config);
    
    // Initialize browser service
    if let Some(browser_service) = &service.browser_service {
        println!("Initializing browser service...");
        browser_service.initialize().await?;
        println!("✅ Browser service initialized\n");
    }
    
    // Test URLs - mix of static and dynamic content
    let test_urls = vec![
        ("https://www.rust-lang.org", "Static site"),
        ("https://twitter.com/rustlang", "SPA (Twitter)"),
        ("https://github.com/rust-lang/rust", "GitHub (may use browser)"),
        ("https://reddit.com/r/rust", "Reddit (SPA)"),
        ("https://example.com", "Simple static page"),
    ];
    
    for (url, description) in test_urls {
        println!("Testing: {} ({})", url, description);
        println!("{}", "-".repeat(60));
        
        match service.generate_preview(url).await {
            Ok(preview) => {
                if let Some(browser_service) = &service.browser_service {
                    let used_browser = browser_service.should_use_browser(url);
                    println!("Used browser: {}", if used_browser { "Yes" } else { "No" });
                }
                
                println!("Title: {}", preview.title.unwrap_or_else(|| "None".to_string()));
                println!(
                    "Description: {}",
                    preview
                        .description
                        .map(|d| {
                            if d.len() > 100 {
                                format!("{}...", &d[..100])
                            } else {
                                d
                            }
                        })
                        .unwrap_or_else(|| "None".to_string())
                );
                
                if let Some(image) = preview.image_url {
                    println!("Image: {}", image);
                }
                
                if let Some(site) = preview.site_name {
                    println!("Site: {}", site);
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
        
        println!("\n");
    }
    
    // Demonstrate custom browser usage policy
    println!("🔧 Testing with different browser policies:\n");
    
    // Always use browser
    let always_config = PreviewServiceConfig::new(100)
        .with_mcp_config(McpConfig {
            enabled: true,
            ..Default::default()
        })
        .with_browser_usage_policy(BrowserUsagePolicy::Always);
    
    let always_service = PreviewService::new_with_config(always_config);
    
    println!("Policy: Always use browser");
    match always_service.generate_preview("https://example.com").await {
        Ok(preview) => {
            println!("✅ Successfully fetched with browser");
            println!("Title: {:?}", preview.title);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Never use browser (standard HTTP fetch)
    let never_config = PreviewServiceConfig::new(100)
        .with_mcp_config(McpConfig {
            enabled: true,
            ..Default::default()
        })
        .with_browser_usage_policy(BrowserUsagePolicy::Never);
    
    let never_service = PreviewService::new_with_config(never_config);
    
    println!("\nPolicy: Never use browser");
    match never_service.generate_preview("https://twitter.com").await {
        Ok(preview) => {
            println!("✅ Successfully fetched without browser");
            println!("Title: {:?}", preview.title);
        }
        Err(e) => println!("⚠️  Expected - SPAs often fail without browser: {}", e),
    }
    
    println!("\n✨ Example completed!");
    
    Ok(())
}