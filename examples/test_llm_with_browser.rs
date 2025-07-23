//! Test LLM extraction with browser rendering
//! This demonstrates how to use browser rendering for JavaScript-heavy sites
//! before extracting structured data with LLM
//!
//! Run with:
//! ```
//! OPENAI_API_KEY=your_key cargo run --example test_llm_with_browser --features "browser llm"
//! ```

#[cfg(all(feature = "browser", feature = "llm"))]
use url_preview::{
    BrowserPreviewService, McpConfig, McpTransport, BrowserUsagePolicy,
    LLMExtractor, LLMExtractorConfig, ContentFormat,
    OpenAIProvider, MockProvider, LLMProvider,
    FetchResult, Preview,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;

#[cfg(not(all(feature = "browser", feature = "llm")))]
fn main() {
    eprintln!("This example requires both 'browser' and 'llm' features to be enabled.");
    eprintln!("Run with: cargo run --example test_llm_with_browser --features \"browser llm\"");
}

// Define structured data for extraction
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebPageInfo {
    /// Page title
    title: String,
    /// Main heading or tagline
    heading: Option<String>,
    /// Page description
    description: Option<String>,
    /// Key features or sections
    features: Vec<String>,
    /// Call-to-action buttons
    cta_buttons: Vec<String>,
    /// Navigation menu items
    nav_items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct TechArticle {
    /// Article title
    title: String,
    /// Author information
    author: Option<String>,
    /// Publication date
    date: Option<String>,
    /// Article summary
    summary: String,
    /// Code examples found
    code_snippets: Vec<String>,
    /// Technologies mentioned
    technologies: Vec<String>,
}

#[cfg(all(feature = "browser", feature = "llm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐🤖 Browser + LLM Integration Test");
    println!("{}", "=".repeat(60));
    
    // Check for API key
    let use_mock = env::var("OPENAI_API_KEY").is_err();
    
    if use_mock {
        println!("⚠️  No OPENAI_API_KEY found, using mock provider");
        println!("   Set OPENAI_API_KEY environment variable to use real extraction\n");
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
    
    // Configure LLM extractor
    let llm_config = LLMExtractorConfig {
        format: ContentFormat::Markdown, // Better for LLMs
        clean_html: true,
        max_content_length: 100_000,
        ..Default::default()
    };
    
    let llm_extractor = LLMExtractor::with_config(provider, llm_config);
    
    // Test cases
    let test_cases = vec![
        ("https://react.dev", "React Documentation (SPA)", true),
        ("https://www.rust-lang.org", "Rust Language Site", false),
        ("https://angular.io", "Angular Framework (SPA)", true),
        ("https://svelte.dev", "Svelte Documentation", true),
    ];
    
    for (url, description, is_spa) in test_cases {
        println!("\n📄 Testing: {} - {}", url, description);
        println!("   SPA: {}", if is_spa { "Yes" } else { "No" });
        println!("{}", "-".repeat(60));
        
        // Step 1: Get preview with browser
        println!("   1️⃣  Fetching with browser...");
        match browser_service.generate_preview(url).await {
            Ok(preview) => {
                println!("   ✅ Browser preview generated:");
                println!("      Title: {}", preview.title.as_deref().unwrap_or("(none)"));
                println!("      Description: {}", 
                    preview.description.as_deref()
                        .map(|d| if d.len() > 80 { 
                            format!("{}...", &d[..80]) 
                        } else { 
                            d.to_string() 
                        })
                        .unwrap_or_else(|| "(none)".to_string())
                );
                
                // Step 2: Extract structured data
                println!("\n   2️⃣  Extracting structured data with LLM...");
                
                // Create a temporary fetcher with the browser-rendered content
                let fetcher = Arc::new(url_preview::Fetcher::new());
                
                match llm_extractor.extract::<WebPageInfo>(url, &fetcher).await {
                    Ok(info) => {
                        println!("   ✅ Structured data extracted:");
                        println!("      Title: {}", info.data.title);
                        if let Some(heading) = info.data.heading {
                            println!("      Heading: {}", heading);
                        }
                        println!("      Features: {} found", info.data.features.len());
                        for (i, feature) in info.data.features.iter().take(3).enumerate() {
                            println!("        {}. {}", i + 1, feature);
                        }
                        println!("      CTA Buttons: {:?}", info.data.cta_buttons);
                        println!("      Nav Items: {} found", info.data.nav_items.len());
                        
                        if let Some(usage) = info.usage {
                            println!("      Token usage: {} prompt, {} completion", 
                                usage.prompt_tokens, usage.completion_tokens);
                        }
                    }
                    Err(e) => println!("   ❌ LLM extraction error: {}", e),
                }
            }
            Err(e) => println!("   ❌ Browser error: {}", e),
        }
    }
    
    // Test with a technical article
    println!("\n\n📚 Testing technical article extraction:");
    println!("{}", "=".repeat(60));
    
    let article_url = "https://blog.rust-lang.org/2024/01/03/Rust-1.75.0.html";
    println!("\nURL: {}", article_url);
    
    // Fetch with browser first
    match browser_service.generate_preview(article_url).await {
        Ok(_) => {
            println!("✅ Page loaded with browser");
            
            let fetcher = Arc::new(url_preview::Fetcher::new());
            match llm_extractor.extract::<TechArticle>(article_url, &fetcher).await {
                Ok(article) => {
                    println!("\n📝 Article extracted:");
                    println!("   Title: {}", article.data.title);
                    if let Some(author) = article.data.author {
                        println!("   Author: {}", author);
                    }
                    if let Some(date) = article.data.date {
                        println!("   Date: {}", date);
                    }
                    println!("   Summary: {}", 
                        if article.data.summary.len() > 150 {
                            format!("{}...", &article.data.summary[..150])
                        } else {
                            article.data.summary.clone()
                        }
                    );
                    println!("   Technologies: {:?}", article.data.technologies);
                    println!("   Code snippets: {} found", article.data.code_snippets.len());
                }
                Err(e) => println!("❌ Extraction error: {}", e),
            }
        }
        Err(e) => println!("❌ Browser error: {}", e),
    }
    
    println!("\n\n🎉 All tests completed!");
    
    Ok(())
}