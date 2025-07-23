//! Test LLM extraction using claude-code-sdk
//! 
//! This example shows how to use the claude-code-sdk to extract structured data
//! from web pages, replacing the built-in LLM providers.
//!
//! Prerequisites:
//! 1. Install Claude Code CLI: npm install -g @anthropic-ai/claude-code
//! 2. Add to Cargo.toml: 
//!    claude-code-sdk = { path = "../rust-claude-code-api/crates/claude-code-sdk-rs" }
//!
//! Run with:
//! ```
//! cargo run --example test_claude_code_sdk
//! ```

use url_preview::{Fetcher, FetchResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::StreamExt;

// Import from claude-code-sdk (you'll need to add this to Cargo.toml)
// use claude_code_sdk::{query, ClaudeCodeOptions, Message, ContentBlock};

// For now, let's create a mock implementation to show the pattern
mod claude_mock {
    use futures::stream::{self, BoxStream};
    
    #[derive(Debug)]
    pub struct ClaudeCodeOptions {
        pub system_prompt: String,
        pub model: String,
    }
    
    impl ClaudeCodeOptions {
        pub fn builder() -> OptionsBuilder {
            OptionsBuilder::default()
        }
    }
    
    #[derive(Default)]
    pub struct OptionsBuilder {
        system_prompt: Option<String>,
        model: Option<String>,
    }
    
    impl OptionsBuilder {
        pub fn system_prompt(mut self, prompt: &str) -> Self {
            self.system_prompt = Some(prompt.to_string());
            self
        }
        
        pub fn model(mut self, model: &str) -> Self {
            self.model = Some(model.to_string());
            self
        }
        
        pub fn build(self) -> ClaudeCodeOptions {
            ClaudeCodeOptions {
                system_prompt: self.system_prompt.unwrap_or_default(),
                model: self.model.unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
            }
        }
    }
    
    #[derive(Debug)]
    pub enum Message {
        Assistant { content: String },
        Result { success: bool },
    }
    
    pub async fn query(
        prompt: String,
        _options: Option<ClaudeCodeOptions>,
    ) -> Result<BoxStream<'static, Result<Message, String>>, String> {
        // Mock implementation
        let messages = vec![
            Ok(Message::Assistant {
                content: r#"{"title": "Example Page", "description": "A test page", "topics": ["testing", "example"]}"#.to_string(),
            }),
            Ok(Message::Result { success: true }),
        ];
        
        Ok(Box::pin(stream::iter(messages)))
    }
}

// Define structured data types for extraction
#[derive(Debug, Serialize, Deserialize)]
struct WebPageInfo {
    title: String,
    description: String,
    topics: Vec<String>,
    main_content: Option<String>,
    author: Option<String>,
    publish_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductInfo {
    name: String,
    price: Option<String>,
    description: String,
    features: Vec<String>,
    availability: bool,
    rating: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArticleAnalysis {
    headline: String,
    summary: String,
    key_points: Vec<String>,
    sentiment: String,
    target_audience: String,
    reading_time_minutes: u32,
}

// Custom LLM extractor using claude-code-sdk
struct ClaudeCodeExtractor {
    system_prompt: String,
    model: String,
}

impl ClaudeCodeExtractor {
    fn new() -> Self {
        Self {
            system_prompt: "You are an expert at extracting structured data from web content. \
                           Always respond with valid JSON that matches the requested schema."
                .to_string(),
            model: "claude-3-opus-20240229".to_string(),
        }
    }
    
    async fn extract<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        html_content: &str,
        schema_description: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        use claude_mock::*;
        
        // Build the extraction prompt
        let prompt = format!(
            "Extract structured data from this webpage.\n\n\
            URL: {}\n\n\
            Expected data structure: {}\n\n\
            HTML Content:\n{}\n\n\
            Return ONLY valid JSON matching the structure.",
            url, schema_description, 
            Self::truncate_content(html_content, 5000)
        );
        
        // Configure Claude options
        let options = ClaudeCodeOptions::builder()
            .system_prompt(&self.system_prompt)
            .model(&self.model)
            .build();
        
        // Query Claude
        let mut messages = query(prompt, Some(options)).await?;
        let mut json_response = String::new();
        
        // Collect the response
        while let Some(msg) = messages.next().await {
            match msg? {
                Message::Assistant { content } => {
                    json_response.push_str(&content);
                }
                Message::Result { .. } => break,
            }
        }
        
        // Parse the JSON response
        serde_json::from_str(&json_response)
            .map_err(|e| format!("Failed to parse JSON: {}", e).into())
    }
    
    fn truncate_content(content: &str, max_chars: usize) -> &str {
        if content.len() <= max_chars {
            content
        } else {
            &content[..max_chars]
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 URL Preview with Claude Code SDK Integration");
    println!("{}", "=".repeat(60));
    
    // Create fetcher and extractor
    let fetcher = Arc::new(Fetcher::new());
    let extractor = ClaudeCodeExtractor::new();
    
    // Test URLs
    let test_urls = vec![
        ("https://www.rust-lang.org/", "Rust Language Homepage"),
        ("https://blog.rust-lang.org/", "Rust Blog"),
        ("https://github.com/rust-lang/rust", "Rust GitHub Repository"),
    ];
    
    // Test 1: Extract general web page information
    println!("\n📄 Test 1: General Web Page Information");
    println!("{}", "-".repeat(40));
    
    for (url, description) in &test_urls {
        println!("\n🔍 Analyzing: {} - {}", url, description);
        
        // Fetch the page
        match fetcher.fetch(url).await {
            Ok(FetchResult::Html(html)) => {
                // Extract structured data
                let schema = r#"{
                    "title": "string",
                    "description": "string", 
                    "topics": ["string"],
                    "main_content": "string (optional)",
                    "author": "string (optional)",
                    "publish_date": "string (optional)"
                }"#;
                
                match extractor.extract::<WebPageInfo>(url, &html, schema).await {
                    Ok(info) => {
                        println!("✅ Extracted:");
                        println!("   Title: {}", info.title);
                        println!("   Description: {}", info.description);
                        println!("   Topics: {:?}", info.topics);
                        if let Some(author) = info.author {
                            println!("   Author: {}", author);
                        }
                    }
                    Err(e) => println!("❌ Extraction error: {}", e),
                }
            }
            Err(e) => println!("❌ Fetch error: {}", e),
        }
    }
    
    // Test 2: Extract as product information
    println!("\n\n🛍️ Test 2: Product Information Extraction");
    println!("{}", "-".repeat(40));
    
    let product_url = "https://www.rust-lang.org/tools/install";
    println!("URL: {}", product_url);
    
    match fetcher.fetch(product_url).await {
        Ok(FetchResult::Html(html)) => {
            let schema = r#"{
                "name": "string",
                "price": "string (optional)",
                "description": "string",
                "features": ["string"],
                "availability": "boolean",
                "rating": "number (optional)"
            }"#;
            
            match extractor.extract::<ProductInfo>(product_url, &html, schema).await {
                Ok(product) => {
                    println!("✅ Product Info:");
                    println!("   Name: {}", product.name);
                    println!("   Description: {}", product.description);
                    println!("   Features: {:?}", product.features);
                    println!("   Available: {}", product.availability);
                }
                Err(e) => println!("❌ Error: {}", e),
            }
        }
        Err(e) => println!("❌ Fetch error: {}", e),
    }
    
    // Test 3: Article analysis
    println!("\n\n📰 Test 3: Article Analysis");
    println!("{}", "-".repeat(40));
    
    let article_url = "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0.html";
    println!("URL: {}", article_url);
    
    match fetcher.fetch(article_url).await {
        Ok(FetchResult::Html(html)) => {
            let schema = r#"{
                "headline": "string",
                "summary": "string",
                "key_points": ["string"],
                "sentiment": "string (positive/negative/neutral)",
                "target_audience": "string",
                "reading_time_minutes": "number"
            }"#;
            
            match extractor.extract::<ArticleAnalysis>(article_url, &html, schema).await {
                Ok(analysis) => {
                    println!("✅ Article Analysis:");
                    println!("   Headline: {}", analysis.headline);
                    println!("   Summary: {}", analysis.summary);
                    println!("   Key Points:");
                    for (i, point) in analysis.key_points.iter().enumerate() {
                        println!("     {}. {}", i + 1, point);
                    }
                    println!("   Sentiment: {}", analysis.sentiment);
                    println!("   Target Audience: {}", analysis.target_audience);
                    println!("   Reading Time: {} minutes", analysis.reading_time_minutes);
                }
                Err(e) => println!("❌ Error: {}", e),
            }
        }
        Err(e) => println!("❌ Fetch error: {}", e),
    }
    
    // Integration tips
    println!("\n\n💡 Integration with url-preview:");
    println!("{}", "=".repeat(60));
    println!("1. Add claude-code-sdk to your Cargo.toml:");
    println!("   [dependencies]");
    println!("   claude-code-sdk = {{ path = \"../rust-claude-code-api/crates/claude-code-sdk-rs\" }}");
    println!("\n2. Install Claude Code CLI:");
    println!("   npm install -g @anthropic-ai/claude-code");
    println!("\n3. Create a custom LLMProvider implementation:");
    println!("   - Implement the LLMProvider trait");
    println!("   - Use claude_code_sdk::query() for generation");
    println!("   - Handle streaming responses");
    println!("\n4. Benefits over OpenAI API:");
    println!("   - Direct Claude access");
    println!("   - Tool use capabilities");
    println!("   - Interactive sessions");
    println!("   - Local CLI integration");
    
    println!("\n🎉 Test completed!");
    
    Ok(())
}