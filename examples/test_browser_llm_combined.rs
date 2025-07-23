//! Test combining browser rendering with LLM extraction
//! This is useful for JavaScript-heavy sites that need rendering before extraction
//!
//! Run with:
//! ```
//! OPENAI_API_KEY=your_key cargo run --example test_browser_llm_combined --features "browser llm"
//! ```

#[cfg(all(feature = "browser", feature = "llm"))]
use url_preview::{
    BrowserPreviewService, McpConfig, McpTransport, BrowserUsagePolicy,
    LLMExtractor, LLMExtractorConfig, ContentFormat,
    OpenAIProvider, MockProvider, LLMProvider,
    FetchResult, Fetcher,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;

#[cfg(not(all(feature = "browser", feature = "llm")))]
fn main() {
    eprintln!("This example requires both 'browser' and 'llm' features to be enabled.");
    eprintln!("Run with: cargo run --example test_browser_llm_combined --features \"browser llm\"");
}

// Define structured data for social media profiles
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SocialMediaProfile {
    /// Username or handle
    username: String,
    /// Display name
    display_name: Option<String>,
    /// Bio or description
    bio: Option<String>,
    /// Number of followers (as string to handle formatting like "1.2K")
    followers: Option<String>,
    /// Number of following
    following: Option<String>,
    /// Verification status
    verified: Option<bool>,
    /// Recent posts or tweets (first few)
    recent_posts: Vec<String>,
}

// E-commerce product from dynamic page
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EcommerceProduct {
    /// Product title
    title: String,
    /// Current price
    price: String,
    /// Original price (if on sale)
    original_price: Option<String>,
    /// Product rating (0-5)
    rating: Option<f32>,
    /// Number of reviews
    review_count: Option<u32>,
    /// Key features
    features: Vec<String>,
    /// Stock status
    in_stock: bool,
    /// Product images URLs
    images: Vec<String>,
}

// News article from SPA
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct NewsArticle {
    /// Article headline
    headline: String,
    /// Subheadline or summary
    summary: Option<String>,
    /// Author(s)
    authors: Vec<String>,
    /// Publication time
    publish_time: Option<String>,
    /// Article categories or tags
    categories: Vec<String>,
    /// Main content (first paragraph)
    lead_paragraph: Option<String>,
}

#[cfg(all(feature = "browser", feature = "llm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐🤖 Browser + LLM Combined Extraction Test");
    println!("{}", "=".repeat(60));
    
    // Check for API key
    let use_mock = env::var("OPENAI_API_KEY").is_err();
    
    if use_mock {
        println!("⚠️  No OPENAI_API_KEY found, using mock provider");
        println!("   Set OPENAI_API_KEY environment variable to use real extraction");
        println!();
    }
    
    // Setup browser service
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
    
    let browser_service = Arc::new(BrowserPreviewService::new(mcp_config, BrowserUsagePolicy::Always));
    
    println!("▶️  Initializing browser service...");
    browser_service.initialize().await?;
    println!("✅ Browser service initialized\n");
    
    // Setup LLM provider
    let provider: Arc<dyn LLMProvider> = if use_mock {
        Arc::new(MockProvider::new())
    } else {
        let api_key = env::var("OPENAI_API_KEY")?;
        Arc::new(OpenAIProvider::new(api_key))
    };
    
    // Configure LLM extractor for Markdown format (better for LLMs)
    let llm_config = LLMExtractorConfig {
        format: ContentFormat::Markdown,
        clean_html: true,
        max_content_length: 100_000,
        ..Default::default()
    };
    
    let llm_extractor = LLMExtractor::with_config(provider, llm_config);
    
    // Test cases for JavaScript-heavy sites
    println!("🧪 Testing JavaScript-heavy sites with browser + LLM:\n");
    
    // Test 1: Social media profile (Twitter/X)
    println!("1️⃣  Extracting Twitter/X profile information:");
    println!("{}", "-".repeat(60));
    
    let twitter_url = "https://twitter.com/rustlang";
    
    // First, fetch with browser to render JavaScript
    match browser_service.fetch_with_browser(twitter_url).await {
        Ok(result) => {
            if let FetchResult::Html(html) = result {
                println!("✅ Page rendered with browser ({} bytes)", html.len());
                
                // Create a mock fetcher that returns the browser-rendered HTML
                let browser_html = html.clone();
                let mock_fetcher = Arc::new(MockFetcher::new(browser_html));
                
                // Extract structured data using LLM
                match llm_extractor.extract::<SocialMediaProfile>(twitter_url, &mock_fetcher).await {
                    Ok(profile) => {
                        println!("✅ Profile extracted:");
                        println!("   Username: @{}", profile.username);
                        if let Some(name) = profile.display_name {
                            println!("   Display Name: {}", name);
                        }
                        if let Some(bio) = profile.bio {
                            println!("   Bio: {}", bio);
                        }
                        if let Some(followers) = profile.followers {
                            println!("   Followers: {}", followers);
                        }
                        println!("   Recent posts: {} found", profile.recent_posts.len());
                    }
                    Err(e) => println!("❌ LLM extraction error: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Browser fetch error: {}", e),
    }
    
    // Test 2: E-commerce product page
    println!("\n\n2️⃣  Extracting e-commerce product information:");
    println!("{}", "-".repeat(60));
    
    // Using a well-known e-commerce site that uses JavaScript
    let product_url = "https://www.apple.com/macbook-pro/";
    
    match browser_service.fetch_with_browser(product_url).await {
        Ok(FetchResult::Html(html)) => {
            println!("✅ Page rendered with browser ({} bytes)", html.len());
            
            let mock_fetcher = Arc::new(MockFetcher::new(html));
            
            match llm_extractor.extract::<EcommerceProduct>(product_url, &mock_fetcher).await {
                Ok(product) => {
                    println!("✅ Product extracted:");
                    println!("   Title: {}", product.title);
                    println!("   Price: {}", product.price);
                    if let Some(orig) = product.original_price {
                        println!("   Original Price: {}", orig);
                    }
                    if let Some(rating) = product.rating {
                        println!("   Rating: {:.1}/5", rating);
                    }
                    println!("   In Stock: {}", product.in_stock);
                    println!("   Features: {} listed", product.features.len());
                    for (i, feature) in product.features.iter().take(3).enumerate() {
                        println!("     {}. {}", i + 1, feature);
                    }
                }
                Err(e) => println!("❌ LLM extraction error: {}", e),
            }
        }
        _ => println!("❌ Failed to fetch page"),
    }
    
    // Test 3: News site (SPA)
    println!("\n\n3️⃣  Extracting news article from SPA:");
    println!("{}", "-".repeat(60));
    
    let news_url = "https://techcrunch.com/";
    
    match browser_service.fetch_with_browser(news_url).await {
        Ok(FetchResult::Html(html)) => {
            println!("✅ Page rendered with browser ({} bytes)", html.len());
            
            let mock_fetcher = Arc::new(MockFetcher::new(html));
            
            match llm_extractor.extract::<NewsArticle>(news_url, &mock_fetcher).await {
                Ok(article) => {
                    println!("✅ Article extracted:");
                    println!("   Headline: {}", article.headline);
                    if let Some(summary) = article.summary {
                        println!("   Summary: {}", summary);
                    }
                    println!("   Authors: {}", article.authors.join(", "));
                    if let Some(time) = article.publish_time {
                        println!("   Published: {}", time);
                    }
                    println!("   Categories: {:?}", article.categories);
                }
                Err(e) => println!("❌ LLM extraction error: {}", e),
            }
        }
        _ => println!("❌ Failed to fetch page"),
    }
    
    // Performance comparison
    println!("\n\n📊 Performance Comparison:");
    println!("{}", "=".repeat(60));
    
    let test_url = "https://react.dev";
    
    // Without browser (will fail or get limited content)
    println!("\n🚫 Without browser rendering:");
    let regular_fetcher = Arc::new(url_preview::Fetcher::new());
    let start = std::time::Instant::now();
    match llm_extractor.extract::<CompanyInfo>(test_url, &regular_fetcher).await {
        Ok(info) => println!("   Extracted: {}", info.name),
        Err(e) => println!("   Error: {}", e),
    }
    let duration_no_browser = start.elapsed();
    println!("   Time: {:?}", duration_no_browser);
    
    // With browser
    println!("\n✅ With browser rendering:");
    let start = std::time::Instant::now();
    match browser_service.fetch_with_browser(test_url).await {
        Ok(FetchResult::Html(html)) => {
            let mock_fetcher = Arc::new(MockFetcher::new(html));
            match llm_extractor.extract::<CompanyInfo>(test_url, &mock_fetcher).await {
                Ok(info) => println!("   Extracted: {}", info.name),
                Err(e) => println!("   Error: {}", e),
            }
        }
        _ => println!("   Failed to fetch"),
    }
    let duration_with_browser = start.elapsed();
    println!("   Time: {:?}", duration_with_browser);
    
    println!("\n\n🎉 All tests completed!");
    
    Ok(())
}

// Simple company info for testing
#[cfg(all(feature = "browser", feature = "llm"))]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CompanyInfo {
    name: String,
    description: String,
}

// Since we can't implement the Fetch trait (not exported), 
// we'll use a different approach with the standard Fetcher