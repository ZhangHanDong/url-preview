//! Example combining browser-based fetching with LLM extraction
//!
//! This example shows how to use browser automation to fetch JavaScript-rendered
//! content and then extract structured data using LLMs.
//!
//! Run with:
//! ```
//! cargo run --example browser_llm_extraction --features "browser llm"
//! ```

#[cfg(all(feature = "browser", feature = "llm"))]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "browser", feature = "llm"))]
use url_preview::{
    BrowserPreviewService, BrowserUsagePolicy, ContentFormat, LLMExtractor, 
    LLMExtractorConfig, McpConfig, MockProvider, PreviewError,
};
#[cfg(all(feature = "browser", feature = "llm"))]
use std::sync::Arc;

#[cfg(not(all(feature = "browser", feature = "llm")))]
fn main() {
    eprintln!("This example requires both 'browser' and 'llm' features to be enabled.");
    eprintln!("Run with: cargo run --example browser_llm_extraction --features \"browser llm\"");
    std::process::exit(1);
}

#[cfg(all(feature = "browser", feature = "llm"))]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct TwitterProfile {
    username: String,
    display_name: String,
    bio: Option<String>,
    follower_count: Option<String>,
    following_count: Option<String>,
    tweet_count: Option<String>,
}

#[cfg(all(feature = "browser", feature = "llm"))]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct RedditPost {
    title: String,
    author: String,
    subreddit: String,
    upvotes: Option<i32>,
    comment_count: Option<i32>,
    content: Option<String>,
}

#[cfg(all(feature = "browser", feature = "llm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐🤖 Browser + LLM Extraction Example\n");
    
    // Set up browser service
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@playwright/mcp@latest".to_string(),
        ],
        ..Default::default()
    };
    
    let browser_service = Arc::new(BrowserPreviewService::new(
        mcp_config,
        BrowserUsagePolicy::Always, // Always use browser for this example
    ));
    
    // Initialize browser
    println!("Initializing browser service...");
    browser_service.initialize().await?;
    println!("✅ Browser initialized\n");
    
    // Set up LLM extractor
    let llm_provider = Arc::new(MockProvider::new());
    let llm_config = LLMExtractorConfig {
        format: ContentFormat::Html,
        clean_html: true,
        max_content_length: 100_000,
        ..Default::default()
    };
    let extractor = LLMExtractor::with_config(llm_provider, llm_config);
    
    // Example 1: Extract data from a Twitter/X profile (SPA)
    println!("🐦 Example 1: Twitter/X Profile Extraction");
    println!("{}", "-".repeat(60));
    
    let twitter_url = "https://twitter.com/rustlang";
    
    // First, use browser to get the rendered content
    println!("Fetching with browser...");
    match browser_service.browser_fetcher.fetch_with_browser(twitter_url).await {
        Ok(html) => {
            println!("✅ Browser fetch successful, content size: {} bytes", html.len());
            
            // Now extract structured data using LLM
            println!("Extracting data with LLM...");
            
            // In a real scenario, you'd pass the HTML to the LLM
            // For this mock example, we'll just show the concept
            let mock_profile = TwitterProfile {
                username: "rustlang".to_string(),
                display_name: "Rust Programming Language".to_string(),
                bio: Some("A language empowering everyone to build reliable and efficient software.".to_string()),
                follower_count: Some("100K+".to_string()),
                following_count: Some("50".to_string()),
                tweet_count: Some("5000+".to_string()),
            };
            
            println!("✅ Extracted profile: {:?}", mock_profile);
        }
        Err(e) => {
            println!("❌ Browser fetch failed: {}", e);
        }
    }
    
    println!("\n");
    
    // Example 2: Extract Reddit post data
    println!("🎯 Example 2: Reddit Post Extraction");
    println!("{}", "-".repeat(60));
    
    let reddit_url = "https://reddit.com/r/rust";
    
    println!("Fetching Reddit with browser...");
    match browser_service.browser_fetcher.fetch_with_browser(reddit_url).await {
        Ok(html) => {
            println!("✅ Browser fetch successful");
            
            // Mock extraction result
            let mock_post = RedditPost {
                title: "What's everyone working on this week?".to_string(),
                author: "rust_moderator".to_string(),
                subreddit: "rust".to_string(),
                upvotes: Some(42),
                comment_count: Some(15),
                content: Some("Weekly thread for Rust projects and discussions.".to_string()),
            };
            
            println!("✅ Extracted post: {:?}", mock_post);
        }
        Err(e) => {
            println!("❌ Browser fetch failed: {}", e);
        }
    }
    
    println!("\n");
    
    // Example 3: Custom extraction with JavaScript evaluation
    println!("⚡ Example 3: Custom JavaScript Extraction");
    println!("{}", "-".repeat(60));
    
    let custom_url = "https://example.com";
    
    // Define custom extraction script
    let extraction_script = r#"
        {
            title: document.title,
            metaDescription: document.querySelector('meta[name="description"]')?.content,
            headingCount: document.querySelectorAll('h1, h2, h3').length,
            linkCount: document.querySelectorAll('a').length,
            hasImages: document.querySelectorAll('img').length > 0
        }
    "#;
    
    println!("Navigating to {}...", custom_url);
    match browser_service.browser_fetcher.extract_with_script::<serde_json::Value>(
        custom_url,
        extraction_script,
    ).await {
        Ok(data) => {
            println!("✅ Custom extraction successful:");
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("❌ Custom extraction failed: {}", e);
        }
    }
    
    println!("\n");
    
    // Example 4: Screenshot + LLM analysis
    println!("📸 Example 4: Screenshot Analysis (Concept)");
    println!("{}", "-".repeat(60));
    
    println!("Taking screenshot of {}", custom_url);
    match browser_service.browser_fetcher.take_screenshot(custom_url).await {
        Ok(screenshot_data) => {
            println!("✅ Screenshot captured: {} bytes", screenshot_data.len());
            println!("📝 Note: With a multi-modal LLM, you could analyze this image directly");
            
            // In a real implementation with multi-modal LLM:
            // let visual_analysis = llm_extractor.analyze_screenshot(screenshot_data).await?;
        }
        Err(e) => {
            println!("❌ Screenshot failed: {}", e);
        }
    }
    
    println!("\n✨ Example completed!");
    
    // Best practices
    println!("\n📚 Best Practices:");
    println!("1. Use browser for SPAs and JavaScript-heavy sites");
    println!("2. Cache browser-fetched content when possible");
    println!("3. Implement proper error handling and fallbacks");
    println!("4. Consider rate limiting for external sites");
    println!("5. Use appropriate content formats for LLM processing");
    
    Ok(())
}