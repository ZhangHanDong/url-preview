//! Simple test of LLM extraction using claude-code-api-rs
//!
//! Prerequisites:
//! 1. Start claude-code-api: RUST_LOG=info claude-code-api
//! 2. Run: cargo run --example simple_claude_api_test --features llm

use url_preview::{LLMExtractor, OpenAIProvider, Fetcher, PreviewError};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    main_topics: Vec<String>,
    key_takeaways: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🧪 Testing LLM extraction with claude-code-api...\n");
    
    // Configure provider for local claude-code-api
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-opus-4-20250514".to_string())
    );
    
    // Create extractor and fetcher
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    // Test URL
    let url = "https://blog.rust-lang.org/";
    println!("Extracting from: {}\n", url);
    
    // Extract structured data
    match extractor.extract::<ArticleInfo>(url, &fetcher).await {
        Ok(result) => {
            println!("✅ Extraction successful!\n");
            println!("Title: {}", result.data.title);
            println!("Summary: {}", result.data.summary);
            
            println!("\nMain Topics:");
            for topic in &result.data.main_topics {
                println!("  • {}", topic);
            }
            
            println!("\nKey Takeaways:");
            for (i, takeaway) in result.data.key_takeaways.iter().enumerate() {
                println!("  {}. {}", i + 1, takeaway);
            }
            
            if let Some(usage) = result.usage {
                println!("\nToken Usage: {} prompt + {} completion = {} total",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            println!("\nMake sure claude-code-api is running:");
            println!("  RUST_LOG=info claude-code-api");
        }
    }
    
    Ok(())
}