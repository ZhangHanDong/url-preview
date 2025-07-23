//! Test LLM-based structured data extraction
//!
//! Run with:
//! ```
//! OPENAI_API_KEY=your_key cargo run --example test_llm_extraction --features llm
//! ```

use url_preview::{
    Fetcher, LLMExtractor, LLMExtractorConfig, ContentFormat, 
    OpenAIProvider, MockProvider, LLMProvider,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;

// Define structured data types to extract
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ProductInfo {
    /// Product name or title
    name: String,
    /// Product price (as string to handle various formats)
    price: Option<String>,
    /// Product description
    description: String,
    /// Whether the product is in stock
    availability: bool,
    /// Product rating (0-5)
    rating: Option<f32>,
    /// Number of reviews
    review_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    /// Article title
    title: String,
    /// Author name(s)
    author: Option<String>,
    /// Publication date
    publish_date: Option<String>,
    /// Article summary or excerpt
    summary: String,
    /// Main topics or tags
    topics: Vec<String>,
    /// Estimated reading time in minutes
    reading_time: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CompanyInfo {
    /// Company name
    name: String,
    /// Company description or tagline
    description: String,
    /// Industry or sector
    industry: Option<String>,
    /// Location/headquarters
    location: Option<String>,
    /// Number of employees (as string for ranges like "1000-5000")
    employee_count: Option<String>,
    /// Key products or services
    products: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LLM Structured Data Extraction Test");
    println!("{}", "=".repeat(60));
    
    // Check for API key
    let use_mock = env::var("OPENAI_API_KEY").is_err();
    
    if use_mock {
        println!("⚠️  No OPENAI_API_KEY found, using mock provider");
        println!("   Set OPENAI_API_KEY environment variable to use real extraction");
        println!();
    }
    
    // Create LLM provider
    let provider: Arc<dyn LLMProvider> = if use_mock {
        Arc::new(MockProvider::new())
    } else {
        let api_key = env::var("OPENAI_API_KEY")?;
        Arc::new(OpenAIProvider::new(api_key))
    };
    
    // Create fetcher
    let fetcher = Arc::new(Fetcher::new());
    
    // Test different content formats
    println!("📋 Testing different content formats:");
    println!("{}", "-".repeat(60));
    
    let formats = vec![
        (ContentFormat::Html, "HTML (raw)"),
        (ContentFormat::Markdown, "Markdown (converted)"),
        (ContentFormat::Text, "Text (cleaned)"),
    ];
    
    for (format, name) in formats {
        let config = LLMExtractorConfig {
            format,
            clean_html: true,
            max_content_length: 50_000,
            ..Default::default()
        };
        
        let extractor = LLMExtractor::with_config(provider.clone(), config);
        
        println!("\n🔧 Format: {}", name);
        
        // Test with a simple product page
        let url = "https://www.rust-lang.org/";
        match extractor.extract::<CompanyInfo>(url, &fetcher).await {
            Ok(info) => {
                println!("✅ Successfully extracted CompanyInfo:");
                println!("   Name: {}", info.data.name);
                println!("   Description: {}", info.data.description);
                if let Some(industry) = info.data.industry {
                    println!("   Industry: {}", industry);
                }
                println!("   Products: {:?}", info.data.products);
                if let Some(usage) = info.usage {
                    println!("   Token usage: {} prompt, {} completion", 
                        usage.prompt_tokens, usage.completion_tokens);
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    }
    
    // Test different extraction types
    println!("\n\n🎯 Testing different extraction schemas:");
    println!("{}", "=".repeat(60));
    
    // Create a new extractor for remaining tests
    let extractor = LLMExtractor::new(provider.clone());
    
    // Test 1: Extract article information
    println!("\n📰 Extracting article information from blog post:");
    let article_url = "https://blog.rust-lang.org/";
    match extractor.extract::<ArticleInfo>(article_url, &fetcher).await {
        Ok(article) => {
            println!("✅ Article extracted:");
            println!("   Title: {}", article.data.title);
            println!("   Author: {}", article.data.author.as_deref().unwrap_or("Unknown"));
            println!("   Summary: {}", 
                if article.data.summary.len() > 100 {
                    format!("{}...", &article.data.summary[..100])
                } else {
                    article.data.summary.clone()
                }
            );
            println!("   Topics: {:?}", article.data.topics);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    // Test 2: Extract product information
    println!("\n🛍️  Extracting product information:");
    let product_url = "https://www.rust-lang.org/tools/install";
    match extractor.extract::<ProductInfo>(product_url, &fetcher).await {
        Ok(product) => {
            println!("✅ Product extracted:");
            println!("   Name: {}", product.data.name);
            println!("   Price: {}", product.data.price.as_deref().unwrap_or("Free"));
            println!("   Available: {}", product.data.availability);
            println!("   Description: {}", 
                if product.data.description.len() > 100 {
                    format!("{}...", &product.data.description[..100])
                } else {
                    product.data.description.clone()
                }
            );
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    // Test with caching
    println!("\n\n💾 Testing with caching:");
    println!("{}", "-".repeat(60));
    
    #[cfg(feature = "cache")]
    {
        use url_preview::Cache;
        let cache = Arc::new(Cache::new(100));
        let cached_extractor = extractor.with_cache(cache);
        
        // First request (cache miss)
        println!("\n1️⃣  First request (should hit API):");
        let start = std::time::Instant::now();
        let _ = cached_extractor.extract::<CompanyInfo>("https://github.com", &fetcher).await?;
        let duration1 = start.elapsed();
        println!("   Time: {:?}", duration1);
        
        // Second request (cache hit)
        println!("\n2️⃣  Second request (should use cache):");
        let start = std::time::Instant::now();
        let _ = cached_extractor.extract::<CompanyInfo>("https://github.com", &fetcher).await?;
        let duration2 = start.elapsed();
        println!("   Time: {:?}", duration2);
        println!("   Speed up: {:.1}x", duration1.as_secs_f64() / duration2.as_secs_f64());
    }
    
    #[cfg(not(feature = "cache"))]
    {
        println!("\n⚠️  Caching test skipped (cache feature not enabled)");
        println!("   Enable with: --features \"llm cache\"");
    }
    
    // Test error handling
    println!("\n\n⚠️  Testing error handling:");
    println!("{}", "-".repeat(60));
    
    // Create a new extractor for error handling test
    let error_test_extractor = LLMExtractor::new(provider);
    let invalid_url = "https://this-domain-definitely-does-not-exist-12345.com";
    match error_test_extractor.extract::<CompanyInfo>(invalid_url, &fetcher).await {
        Ok(_) => println!("❓ Unexpected success"),
        Err(e) => println!("✅ Expected error: {}", e),
    }
    
    println!("\n\n🎉 All tests completed!");
    
    Ok(())
}