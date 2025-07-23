//! Simple example of using claude-code-sdk with url-preview
//!
//! Run with: cargo run --example simple_claude_code_example --features llm

use url_preview::{LLMExtractor, ClaudeCodeProvider, PreviewError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ArticleInfo {
    title: String,
    summary: String,
    key_points: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    // Create Claude Code provider
    let provider = ClaudeCodeProvider::new();
    
    // Create LLM extractor
    let extractor = LLMExtractor::with_provider(Box::new(provider));
    
    // Extract structured data from a URL
    let url = "https://blog.rust-lang.org/";
    let result = extractor.extract::<ArticleInfo>(url).await?;
    
    // Use the extracted data
    println!("Title: {}", result.data.title);
    println!("Summary: {}", result.data.summary);
    println!("Key Points:");
    for point in &result.data.key_points {
        println!("  • {}", point);
    }
    
    Ok(())
}