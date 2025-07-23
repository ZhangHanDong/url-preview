//! Example of LLM-based structured data extraction
//!
//! This example demonstrates how to extract structured data from web pages
//! using LLMs (Large Language Models).
//!
//! Run with:
//! ```
//! cargo run --example llm_extraction --features llm
//! ```

#[cfg(feature = "llm")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "llm")]
use url_preview::{
    ContentFormat, Fetcher, LLMExtractor, LLMExtractorConfig, MockProvider, PreviewError,
};
#[cfg(feature = "llm")]
use std::sync::Arc;

#[cfg(not(feature = "llm"))]
fn main() {
    eprintln!("This example requires the 'llm' feature to be enabled.");
    eprintln!("Run with: cargo run --example llm_extraction --features llm");
    std::process::exit(1);
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct ProductInfo {
    name: String,
    price: Option<String>,
    description: String,
    availability: bool,
    rating: Option<f32>,
}

#[cfg(feature = "llm")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct ArticleInfo {
    title: String,
    author: Option<String>,
    publish_date: Option<String>,
    summary: String,
    tags: Vec<String>,
}

#[cfg(feature = "llm")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LLM-based Data Extraction Example\n");
    
    // Create a mock LLM provider for demonstration
    // In production, use OpenAIProvider or AnthropicProvider
    let provider = Arc::new(MockProvider::new());
    
    // Configure the extractor
    let config = LLMExtractorConfig {
        format: ContentFormat::Html,
        clean_html: true,
        max_content_length: 50_000,
        ..Default::default()
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    let fetcher = Fetcher::new();
    
    // Example 1: Extract product information
    println!("📦 Example 1: Extracting Product Information");
    println!("{}", "-".repeat(60));
    
    let product_url = "https://www.rust-lang.org";
    match extractor.extract::<ProductInfo>(product_url, &fetcher).await {
        Ok(result) => {
            println!("✅ Extraction successful!");
            println!("Model used: {}", result.model);
            println!("Product: {:?}", result.data);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    println!("\n");
    
    // Example 2: Extract article information
    println!("📄 Example 2: Extracting Article Information");
    println!("{}", "-".repeat(60));
    
    let article_url = "https://blog.rust-lang.org";
    match extractor.extract::<ArticleInfo>(article_url, &fetcher).await {
        Ok(result) => {
            println!("✅ Extraction successful!");
            println!("Model used: {}", result.model);
            println!("Article: {:?}", result.data);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    println!("\n");
    
    // Example 3: Different content formats
    println!("🔄 Example 3: Testing Different Content Formats");
    println!("{}", "-".repeat(60));
    
    let formats = vec![
        (ContentFormat::Html, "HTML"),
        (ContentFormat::Markdown, "Markdown"),
        (ContentFormat::Text, "Plain Text"),
    ];
    
    for (format, name) in formats {
        println!("\nTesting with {} format:", name);
        
        let config = LLMExtractorConfig {
            format: format.clone(),
            clean_html: true,
            max_content_length: 10_000,
            ..Default::default()
        };
        
        let format_extractor = LLMExtractor::with_config(
            Arc::new(MockProvider::new()),
            config,
        );
        
        match format_extractor.extract::<ProductInfo>("https://example.com", &fetcher).await {
            Ok(result) => {
                println!("✅ {} extraction successful", name);
                println!("Data: {:?}", result.data);
            }
            Err(e) => {
                println!("❌ {} extraction failed: {}", name, e);
            }
        }
    }
    
    // Example 4: Custom schema
    #[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
    struct CustomData {
        main_heading: String,
        links: Vec<String>,
        has_images: bool,
    }
    
    println!("\n\n🎯 Example 4: Custom Schema Extraction");
    println!("{}", "-".repeat(60));
    
    match extractor.extract::<CustomData>("https://www.rust-lang.org", &fetcher).await {
        Ok(result) => {
            println!("✅ Custom extraction successful!");
            println!("Data: {:?}", result.data);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    println!("\n✨ Example completed!");
    
    // Note about real LLM providers
    println!("\n📝 Note: This example uses a mock provider for demonstration.");
    println!("For real extraction, configure with OpenAI or Anthropic:");
    println!("```rust");
    println!("// OpenAI");
    println!("let provider = Arc::new(OpenAIProvider::new(\"your-api-key\".to_string()));");
    println!("");
    println!("// Anthropic");
    println!("let provider = Arc::new(AnthropicProvider::new(\"your-api-key\".to_string()));");
    println!("```");
    
    Ok(())
}