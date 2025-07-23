//! Debug test for MCP client
//!
//! Run with:
//! ```
//! RUST_LOG=debug cargo run --example test_mcp_debug --features "browser logging"
//! ```

#[cfg(feature = "browser")]
use url_preview::{McpClient, McpConfig, McpTransport};
use std::time::Duration;

#[cfg(not(feature = "browser"))]
fn main() {
    eprintln!("This example requires the 'browser' feature to be enabled.");
    eprintln!("Run with: cargo run --example test_mcp_debug --features browser");
}

#[cfg(feature = "browser")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    #[cfg(feature = "logging")]
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    
    println!("🔍 MCP Debug Test");
    println!("{}", "=".repeat(60));
    
    // Configure MCP client
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
    
    println!("📝 MCP Configuration:");
    println!("   Command: {:?}", mcp_config.server_command);
    println!("   Transport: {:?}", mcp_config.transport);
    println!("   Timeout: {}s", mcp_config.browser_timeout);
    
    // Create and start MCP client
    let mcp_client = McpClient::new(mcp_config);
    
    println!("\n▶️  Starting MCP server...");
    match mcp_client.start().await {
        Ok(_) => println!("✅ MCP server started successfully!"),
        Err(e) => {
            println!("❌ Failed to start MCP server: {}", e);
            return Err(e.into());
        }
    }
    
    // Wait a bit more for the server to fully initialize
    println!("\n⏳ Waiting for server to fully initialize...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Test navigation
    println!("\n🌐 Testing navigation to example.com...");
    match mcp_client.navigate("https://example.com").await {
        Ok(_) => println!("✅ Navigation successful!"),
        Err(e) => println!("❌ Navigation failed: {}", e),
    }
    
    // Test page content extraction
    println!("\n📄 Testing page content extraction...");
    match mcp_client.get_page_text().await {
        Ok(text) => {
            println!("✅ Page text extracted!");
            println!("   First 100 chars: {}...", &text[..text.len().min(100)]);
        }
        Err(e) => println!("❌ Failed to extract page text: {}", e),
    }
    
    // Test JavaScript evaluation
    println!("\n🔧 Testing JavaScript evaluation...");
    match mcp_client.evaluate("() => document.title").await {
        Ok(result) => println!("✅ Page title: {:?}", result),
        Err(e) => println!("❌ Failed to evaluate JavaScript: {}", e),
    }
    
    // Stop the server
    println!("\n🛑 Stopping MCP server...");
    mcp_client.stop().await?;
    println!("✅ MCP server stopped.");
    
    println!("\n🎯 Debug test complete!");
    
    Ok(())
}