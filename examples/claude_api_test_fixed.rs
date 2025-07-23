//! Fixed version of claude-code-api test with proper timeout handling
//!
//! Prerequisites:
//! 1. Start claude-code-api with extended timeout:
//!    CLAUDE_CODE__CLAUDE__TIMEOUT_SECONDS=120 RUST_LOG=info claude-code-api
//! 2. Run: cargo run --example claude_api_test_fixed --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SimpleArticleInfo {
    title: String,
    summary: String,
    topics: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🧪 Claude API Test (Fixed Version)\n");
    
    // Check if claude-code-api is running
    match reqwest::get("http://localhost:8080/health").await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Connected to claude-code-api\n");
        }
        _ => {
            println!("⚠️  Please start claude-code-api with extended timeout:");
            println!("   CLAUDE_CODE__CLAUDE__TIMEOUT_SECONDS=120 \\");
            println!("   CLAUDE_CODE__FILE_ACCESS__SKIP_PERMISSIONS=true \\");
            println!("   RUST_LOG=info claude-code-api\n");
            return Ok(());
        }
    }
    
    // Configure provider with timeout handling
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    // Use a faster model for better response times
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-3-haiku-20240307".to_string())
    );
    
    // Configure extractor with smaller content limit
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Text, // Use plain text for faster processing
        clean_html: true,
        max_content_length: 10_000, // Smaller limit for faster processing
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, extractor_config);
    let fetcher = Fetcher::new();
    
    // Test with a simpler URL first
    let test_url = "https://www.rust-lang.org/";
    println!("Testing with: {}\n", test_url);
    
    println!("⏳ Extracting (this may take 10-30 seconds)...\n");
    
    match extractor.extract::<SimpleArticleInfo>(test_url, &fetcher).await {
        Ok(result) => {
            println!("✅ Extraction successful!\n");
            println!("Title: {}", result.data.title);
            println!("Summary: {}", result.data.summary);
            
            println!("\nTopics:");
            for topic in &result.data.topics {
                println!("  • {}", topic);
            }
            
            if let Some(usage) = result.usage {
                println!("\nToken Usage: {} + {} = {}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            
            println!("\n💡 Troubleshooting tips:");
            println!("1. Increase timeout: CLAUDE_CODE__CLAUDE__TIMEOUT_SECONDS=300");
            println!("2. Use simpler schema with fewer fields");
            println!("3. Use claude-3-haiku model for faster responses");
            println!("4. Reduce max_content_length in extractor config");
            println!("5. Check claude-code-api logs for detailed errors");
        }
    }
    
    Ok(())
}