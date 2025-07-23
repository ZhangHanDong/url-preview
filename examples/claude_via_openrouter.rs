//! Example of using Claude via OpenRouter (OpenAI-compatible API)
//!
//! OpenRouter provides access to Claude and other models through an OpenAI-compatible API.
//! This example shows how to configure the OpenAI provider to use Claude.
//!
//! Prerequisites:
//! 1. Sign up at https://openrouter.ai/
//! 2. Get your API key
//! 3. Run with: OPENROUTER_API_KEY=your_key cargo run --example claude_via_openrouter --features llm

use url_preview::{
    Fetcher, LLMExtractor, LLMExtractorConfig, ContentFormat,
    LLMProvider, PreviewError,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;
use async_trait::async_trait;
use serde_json::Value;

// Custom provider for OpenRouter
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "anthropic/claude-3-opus".to_string(), // Claude 3 Opus via OpenRouter
            client: reqwest::Client::new(),
        }
    }
    
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }
    
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        // Build the extraction prompt
        let system_content = format!(
            "You are a helpful assistant that extracts structured data from web content. \
             Extract information according to this JSON schema:\n\n{}\n\n\
             Return ONLY valid JSON that matches the schema, with no additional text.",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        
        let messages = vec![
            serde_json::json!({
                "role": "system",
                "content": system_content
            }),
            serde_json::json!({
                "role": "user",
                "content": format!("Extract structured data from:\n\n{}", prompt)
            })
        ];
        
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.1,
            "max_tokens": 4096,
        });
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/your-app") // Optional but recommended
            .header("X-Title", "URL Preview Extractor") // Optional
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "OpenRouter".to_string(),
                message: format!("Request failed: {}", e),
            })?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PreviewError::ExternalServiceError {
                service: "OpenRouter".to_string(),
                message: format!("API error: {}", error_text),
            });
        }
        
        let response_json: Value = response.json().await
            .map_err(|e| PreviewError::ParseError(format!("Failed to parse response: {}", e)))?;
        
        // Extract the content from the response
        if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
            serde_json::from_str(content)
                .map_err(|e| PreviewError::ParseError(format!("Failed to parse extracted JSON: {}", e)))
        } else {
            Err(PreviewError::ExtractError("No content in response".to_string()))
        }
    }
}

// Example structured data
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BlogPost {
    title: String,
    author: Option<String>,
    date: Option<String>,
    summary: String,
    main_topics: Vec<String>,
    tone: String, // professional, casual, technical, etc.
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Using Claude via OpenRouter Example");
    println!("{}", "=".repeat(60));
    
    // Get API key
    let api_key = match env::var("OPENROUTER_API_KEY").or_else(|_| env::var("OPENAI_API_KEY")) {
        Ok(key) => {
            println!("✅ API key found");
            key
        }
        Err(_) => {
            eprintln!("❌ No OPENROUTER_API_KEY found!");
            eprintln!("\nTo use this example:");
            eprintln!("1. Sign up at https://openrouter.ai/");
            eprintln!("2. Get your API key");
            eprintln!("3. Set: export OPENROUTER_API_KEY=sk-or-...");
            eprintln!("\nAlternatively, you can use other Claude-compatible services:");
            eprintln!("- Helicone: https://helicone.ai/");
            eprintln!("- Custom proxy servers");
            return Ok(());
        }
    };
    
    // Available Claude models on OpenRouter
    println!("\n📋 Available Claude models on OpenRouter:");
    println!("   • anthropic/claude-3-opus (most capable)");
    println!("   • anthropic/claude-3-sonnet (balanced)");
    println!("   • anthropic/claude-3-haiku (fastest)");
    println!("   • anthropic/claude-2.1");
    println!("   • anthropic/claude-instant-1.2");
    
    // Create provider
    let provider = Arc::new(
        OpenRouterProvider::new(api_key)
            .with_model("anthropic/claude-3-sonnet") // Using Sonnet for balance
    );
    
    // Create extractor
    let config = LLMExtractorConfig {
        format: ContentFormat::Markdown,
        clean_html: true,
        max_content_length: 100_000,
        ..Default::default()
    };
    
    let extractor = LLMExtractor::with_config(provider as Arc<dyn LLMProvider>, config);
    let fetcher = Arc::new(Fetcher::new());
    
    // Test extraction
    println!("\n\n🔍 Testing extraction with Claude:");
    println!("{}", "-".repeat(60));
    
    let url = "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0.html";
    println!("URL: {}", url);
    
    match extractor.extract::<BlogPost>(url, &fetcher).await {
        Ok(result) => {
            println!("\n✅ Extraction successful!");
            println!("   Title: {}", result.data.title);
            if let Some(author) = result.data.author {
                println!("   Author: {}", author);
            }
            if let Some(date) = result.data.date {
                println!("   Date: {}", date);
            }
            println!("   Tone: {}", result.data.tone);
            println!("   Summary: {}", 
                if result.data.summary.len() > 100 {
                    format!("{}...", &result.data.summary[..100])
                } else {
                    result.data.summary.clone()
                }
            );
            println!("   Topics: {}", result.data.main_topics.join(", "));
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            println!("\nTroubleshooting:");
            println!("1. Check your API key is valid");
            println!("2. Ensure you have credits on OpenRouter");
            println!("3. Check the model name is correct");
        }
    }
    
    println!("\n\n💡 Other Claude API Options:");
    println!("{}", "=".repeat(60));
    
    println!("1. **Direct Anthropic API** (when Rust SDK is available):");
    println!("   ```rust");
    println!("   let client = anthropic::Client::new(api_key);");
    println!("   ```\n");
    
    println!("2. **Via Helicone** (analytics + caching):");
    println!("   ```rust");
    println!("   client.post(\"https://api.helicone.ai/v1/chat/completions\")");
    println!("      .header(\"Helicone-Auth\", \"Bearer YOUR_HELICONE_KEY\")");
    println!("      .header(\"Helicone-Target-Provider\", \"Anthropic\")");
    println!("   ```\n");
    
    println!("3. **Via LiteLLM Proxy**:");
    println!("   ```bash");
    println!("   litellm --model claude-3-opus");
    println!("   # Then use http://localhost:8000 as base URL");
    println!("   ```");
    
    println!("\n🎉 Example completed!");
    
    Ok(())
}