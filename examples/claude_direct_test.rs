//! Direct test without health check
//!
//! Run: cargo run --example claude_direct_test --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SimpleInfo {
    title: String,
    description: String,
    keywords: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🧪 Direct Claude API Test\n");
    
    // Skip health check - assume service is running
    println!("Using claude-code-api at http://localhost:8080\n");
    
    // Configure provider
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-3-haiku-20240307".to_string())
    );
    
    // Simple configuration
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Text,
        clean_html: true,
        max_content_length: 5_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, extractor_config);
    let fetcher = Fetcher::new();
    
    // Test
    let url = "https://www.rust-lang.org/";
    println!("Extracting from: {}\n", url);
    
    match extractor.extract::<SimpleInfo>(url, &fetcher).await {
        Ok(result) => {
            println!("✅ Success!\n");
            println!("Title: {}", result.data.title);
            println!("Description: {}", result.data.description);
            println!("Keywords: {}", result.data.keywords.join(", "));
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("\nPossible issues:");
            println!("1. Make sure claude-code-api is running");
            println!("2. Check if it's on port 8080");
            println!("3. Try increasing timeout if needed");
        }
    }
    
    Ok(())
}