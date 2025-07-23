//! Optimized extraction example for claude-code-api
//! 
//! This example is optimized to work within Claude's constraints:
//! - Simpler schemas for faster processing
//! - Smaller content limits
//! - Using Haiku model for speed
//!
//! Run: cargo run --example claude_optimized_extraction --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::time::Instant;

// Simplified schemas optimized for Claude
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageSummary {
    title: String,
    description: String,
    main_topic: String,
    key_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ProjectInfo {
    name: String,
    description: String,
    language: String,
    key_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BlogPostList {
    blog_title: String,
    recent_posts: Vec<BlogPost>,
    main_themes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BlogPost {
    title: String,
    date: Option<String>,
}

fn create_optimized_extractor(model: &str) -> LLMExtractor {
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(config, model.to_string())
    );
    
    // Optimized configuration
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Text, // Faster than HTML/Markdown
        clean_html: true,
        max_content_length: 5_000, // Small limit for speed
        model_params: Default::default(),
    };
    
    LLMExtractor::with_config(provider, extractor_config)
}

async fn test_page_summary() -> Result<(), PreviewError> {
    println!("\n📄 Test 1: Simple Page Summary (Haiku model)");
    println!("{}", "=".repeat(50));
    
    let extractor = create_optimized_extractor("claude-3-haiku-20240307");
    let fetcher = Fetcher::new();
    
    let url = "https://www.rust-lang.org/";
    println!("URL: {}", url);
    
    let start = Instant::now();
    match extractor.extract::<PageSummary>(url, &fetcher).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("\n✅ Success in {:.1}s", elapsed.as_secs_f64());
            
            println!("\nTitle: {}", result.data.title);
            println!("Description: {}", result.data.description);
            println!("Main Topic: {}", result.data.main_topic);
            
            println!("\nKey Points:");
            for point in &result.data.key_points {
                println!("  • {}", point);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    Ok(())
}

async fn test_project_info() -> Result<(), PreviewError> {
    println!("\n\n🔧 Test 2: GitHub Project Info (Sonnet model)");
    println!("{}", "=".repeat(50));
    
    let extractor = create_optimized_extractor("claude-3-5-sonnet-20241022");
    let fetcher = Fetcher::new();
    
    let url = "https://github.com/rust-lang/rust";
    println!("URL: {}", url);
    
    let start = Instant::now();
    match extractor.extract::<ProjectInfo>(url, &fetcher).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("\n✅ Success in {:.1}s", elapsed.as_secs_f64());
            
            println!("\nProject: {}", result.data.name);
            println!("Description: {}", result.data.description);
            println!("Language: {}", result.data.language);
            
            println!("\nKey Features:");
            for feature in &result.data.key_features {
                println!("  • {}", feature);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    Ok(())
}

async fn test_blog_extraction() -> Result<(), PreviewError> {
    println!("\n\n📰 Test 3: Blog Post List (with caching)");
    println!("{}", "=".repeat(50));
    
    let extractor = create_optimized_extractor("claude-3-haiku-20240307");
    let fetcher = Fetcher::new();
    
    let url = "https://blog.rust-lang.org/";
    println!("URL: {}", url);
    
    // First request (will be cached)
    println!("\nFirst request:");
    let start = Instant::now();
    match extractor.extract::<BlogPostList>(url, &fetcher).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("✅ Success in {:.1}s", elapsed.as_secs_f64());
            
            println!("\nBlog: {}", result.data.blog_title);
            println!("\nRecent Posts ({}):", result.data.recent_posts.len());
            for post in &result.data.recent_posts[..5.min(result.data.recent_posts.len())] {
                print!("  • {}", post.title);
                if let Some(date) = &post.date {
                    print!(" ({})", date);
                }
                println!();
            }
            
            println!("\nMain Themes:");
            for theme in &result.data.main_themes {
                println!("  • {}", theme);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Second request (should be faster if cached)
    println!("\nSecond request (testing cache):");
    let start = Instant::now();
    match extractor.extract::<BlogPostList>(url, &fetcher).await {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("✅ Success in {:.1}s (cached)", elapsed.as_secs_f64());
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Claude Optimized Extraction Demo");
    println!("{}", "=".repeat(60));
    
    // Check server
    match reqwest::get("http://localhost:8080/health").await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Connected to claude-code-api\n");
        }
        Ok(resp) => {
            println!("⚠️  claude-code-api returned status: {}", resp.status());
            println!("   The service might be running but /health endpoint not available");
            println!("   Continuing anyway...\n");
        }
        Err(e) => {
            println!("⚠️  Cannot connect to claude-code-api at http://localhost:8080");
            println!("   Error: {}", e);
            println!("   Make sure claude-code-api is running on port 8080");
            println!("   Or continue anyway if it's on a different port...\n");
            
            // Ask user if they want to continue
            println!("Continue anyway? (y/n): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            if !input.trim().eq_ignore_ascii_case("y") {
                return Ok(());
            }
        }
    }
    
    // Run tests
    test_page_summary().await?;
    test_project_info().await?;
    test_blog_extraction().await?;
    
    println!("\n\n🎯 Optimization Tips:");
    println!("1. Use Haiku model for simple extractions (fastest)");
    println!("2. Use Sonnet for complex analysis (balanced)");
    println!("3. Keep schemas simple with fewer fields");
    println!("4. Limit content to 5-10KB for best performance");
    println!("5. Use PlainText format instead of HTML/Markdown");
    println!("6. Enable caching for repeated requests");
    
    Ok(())
}