//! Working example using correct model names and response handling
//!
//! Prerequisites:
//! 1. Start claude-code-api: RUST_LOG=info claude-code-api
//! 2. Run: cargo run --example claude_api_working --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebPageInfo {
    title: String,
    description: String,
    main_topics: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🧪 Claude API Working Example\n");
    
    // Configure provider
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    // Use the correct model name from claude-code-api
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-3-5-haiku-20241022".to_string())
    );
    
    // Configure extractor
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Text,
        clean_html: true,
        max_content_length: 5_000, // Keep it small
        model_params: Default::default(),
    };
    
    let _extractor = LLMExtractor::with_config(provider, extractor_config.clone());
    let fetcher = Fetcher::new();
    
    // Test with different models
    let models = [
        ("claude-3-5-haiku-20241022", "Haiku (fast)"),
        ("claude-3-5-sonnet-20241022", "Sonnet (balanced)"),
        ("opus", "Opus (powerful)"),
    ];
    
    for (model, desc) in &models {
        println!("\n📊 Testing with {} model", desc);
        println!("{}", "=".repeat(50));
        
        // Create provider with specific model
        let provider = Arc::new(
            OpenAIProvider::from_config(
                async_openai::config::OpenAIConfig::new()
                    .with_api_base("http://localhost:8080/v1")
                    .with_api_key("not-needed"),
                model.to_string()
            )
        );
        
        let extractor = LLMExtractor::with_config(provider, extractor_config.clone());
        
        let url = "https://www.rust-lang.org/";
        println!("URL: {}", url);
        
        let start = std::time::Instant::now();
        match extractor.extract::<WebPageInfo>(url, &fetcher).await {
            Ok(result) => {
                let elapsed = start.elapsed();
                println!("\n✅ Success in {:.1}s", elapsed.as_secs_f64());
                
                println!("\nTitle: {}", result.data.title);
                println!("Description: {}", result.data.description);
                
                println!("\nMain Topics:");
                for topic in &result.data.main_topics {
                    println!("  • {}", topic);
                }
                
                if let Some(usage) = result.usage {
                    println!("\nTokens: {} + {} = {}",
                        usage.prompt_tokens, 
                        usage.completion_tokens, 
                        usage.total_tokens
                    );
                }
            }
            Err(e) => {
                println!("\n❌ Error with {}: {}", model, e);
                
                // Provide specific guidance based on error
                if e.to_string().contains("No function call") {
                    println!("\n💡 This model might not support function calling.");
                    println!("   The response format may need adjustment.");
                } else if e.to_string().contains("Timeout") {
                    println!("\n💡 Model took too long. Try:");
                    println!("   - Increasing timeout in claude-code-api");
                    println!("   - Using a simpler schema");
                    println!("   - Reducing content size");
                }
            }
        }
    }
    
    println!("\n\n📝 Summary:");
    println!("• Use the correct model names (check claude-code-api source)");
    println!("• Haiku models are fastest but may have limitations");
    println!("• Sonnet provides good balance of speed and capability");
    println!("• Opus is most capable but slowest");
    
    Ok(())
}