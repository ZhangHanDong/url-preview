//! Example handling Claude's text responses (not function calls)
//!
//! This demonstrates how to work with Claude when it returns
//! plain text JSON instead of function calls.
//!
//! Run: cargo run --example claude_text_response_handler --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat, LLMProvider, ExtractionResult
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleData {
    title: String,
    summary: String,
    key_points: Vec<String>,
}

/// Custom provider wrapper that handles Claude's text responses
struct ClaudeTextProvider {
    inner: OpenAIProvider,
}

impl ClaudeTextProvider {
    fn new(base_url: &str, model: &str) -> Self {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key("not-needed");
        
        Self {
            inner: OpenAIProvider::from_config(config, model.to_string()),
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeTextProvider {
    async fn extract_structured_data<T: serde::de::DeserializeOwned + JsonSchema + Send>(
        &self,
        content: &str,
        format: ContentFormat,
    ) -> Result<ExtractionResult<T>, PreviewError> {
        // First try the normal extraction
        match self.inner.extract_structured_data::<T>(content, format).await {
            Ok(result) => Ok(result),
            Err(e) if e.to_string().contains("No function call") => {
                // Claude might have returned plain text JSON
                // Try to parse it directly from the response
                println!("⚠️  Attempting to parse plain text response...");
                
                // For now, we'll return the error
                // In a real implementation, you'd parse the text response
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🧪 Claude Text Response Handler Example\n");
    
    // Test with standard provider first
    println!("1️⃣ Testing with standard OpenAI provider:");
    test_standard_provider().await?;
    
    // Test with custom provider
    println!("\n2️⃣ Testing with custom text-aware provider:");
    test_custom_provider().await?;
    
    // Demonstrate direct API call
    println!("\n3️⃣ Testing direct API call (for debugging):");
    test_direct_api().await?;
    
    Ok(())
}

async fn test_standard_provider() -> Result<(), PreviewError> {
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-3-5-haiku-20241022".to_string())
    );
    
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Text,
        clean_html: true,
        max_content_length: 3_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, extractor_config);
    let fetcher = Fetcher::new();
    
    match extractor.extract::<ArticleData>("https://www.rust-lang.org/", &fetcher).await {
        Ok(result) => {
            println!("✅ Success!");
            println!("Title: {}", result.data.title);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}

async fn test_custom_provider() -> Result<(), PreviewError> {
    let provider = Arc::new(
        ClaudeTextProvider::new("http://localhost:8080/v1", "claude-3-5-haiku-20241022")
    );
    
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    match extractor.extract::<ArticleData>("https://www.rust-lang.org/", &fetcher).await {
        Ok(result) => {
            println!("✅ Success!");
            println!("Title: {}", result.data.title);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}

async fn test_direct_api() -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;
    
    let client = reqwest::Client::new();
    
    // Create a simple completion request
    let request = json!({
        "model": "claude-3-5-haiku-20241022",
        "messages": [{
            "role": "user",
            "content": "Extract the following from this text: title, summary (one sentence), and 3 key points. Return as JSON.\n\nText: Rust is a multi-paradigm programming language focused on performance and safety. It achieves memory safety without garbage collection through its ownership system. Rust is syntactically similar to C++ but provides memory safety without using garbage collection."
        }],
        "max_tokens": 500,
        "temperature": 0.0
    });
    
    println!("📤 Sending request to claude-code-api...");
    
    let response = client
        .post("http://localhost:8080/v1/chat/completions")
        .header("Authorization", "Bearer not-needed")
        .json(&request)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    println!("📥 Response status: {}", status);
    
    if status.is_success() {
        // Try to parse the response
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(json) => {
                println!("✅ Response JSON:");
                println!("{}", serde_json::to_string_pretty(&json)?);
                
                // Check if it has function calls or just text
                if let Some(choices) = json["choices"].as_array() {
                    if let Some(first) = choices.first() {
                        if first["message"]["function_call"].is_object() {
                            println!("\n✅ Response includes function call");
                        } else if first["message"]["content"].is_string() {
                            println!("\n⚠️  Response is plain text (no function call)");
                            println!("Content: {}", first["message"]["content"]);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to parse JSON: {}", e);
                println!("Raw response: {}", text);
            }
        }
    } else {
        println!("❌ Error response: {}", text);
    }
    
    Ok(())
}