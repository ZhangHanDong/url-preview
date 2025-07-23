//! Test LLM extraction using Claude API
//! 
//! Claude doesn't have a direct OpenAI-compatible endpoint, but you can use
//! proxy services or the Anthropic SDK directly.
//!
//! For this example, we'll show how to configure for common Claude proxy services.
//!
//! Run with:
//! ```
//! ANTHROPIC_API_KEY=your_key cargo run --example test_claude_api --features llm
//! ```

use url_preview::{
    Fetcher, LLMExtractor, LLMExtractorConfig, ContentFormat, 
    AnthropicProvider, MockProvider, LLMProvider,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;

// Define a comprehensive extraction schema
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebPageAnalysis {
    /// Page title
    title: String,
    /// Page type (blog, documentation, product, landing, etc.)
    page_type: String,
    /// Main purpose or goal of the page
    purpose: String,
    /// Primary call-to-action
    primary_cta: Option<String>,
    /// Key sections or components
    sections: Vec<PageSection>,
    /// Technical details found
    technical_info: TechnicalInfo,
    /// SEO metadata
    seo_data: SeoData,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PageSection {
    /// Section title or heading
    title: String,
    /// Brief description of section content
    description: String,
    /// Section importance (high, medium, low)
    importance: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct TechnicalInfo {
    /// Programming languages mentioned
    languages: Vec<String>,
    /// Frameworks or libraries mentioned
    frameworks: Vec<String>,
    /// Code examples present
    has_code_examples: bool,
    /// API documentation present
    has_api_docs: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SeoData {
    /// Meta description
    meta_description: Option<String>,
    /// Main keywords
    keywords: Vec<String>,
    /// Estimated content quality (1-10)
    content_quality_score: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Claude API LLM Extraction Test");
    println!("{}", "=".repeat(60));
    
    // Check for API key
    let api_key = env::var("ANTHROPIC_API_KEY").or_else(|_| env::var("CLAUDE_API_KEY"));
    
    let provider: Arc<dyn LLMProvider> = match api_key {
        Ok(key) => {
            println!("✅ Anthropic API key found");
            println!("📝 Note: The Anthropic provider in this library is a placeholder.");
            println!("   For production use, you would need to:");
            println!("   1. Implement the full Anthropic SDK integration");
            println!("   2. Or use a Claude-to-OpenAI proxy service");
            println!("   3. Or use the official Anthropic SDK directly\n");
            
            // For now, we'll use the mock provider to demonstrate
            println!("🔄 Using mock provider for demonstration...\n");
            Arc::new(MockProvider::new())
            
            // In a real implementation, you would use:
            // Arc::new(AnthropicProvider::new(key))
        }
        Err(_) => {
            println!("⚠️  No ANTHROPIC_API_KEY or CLAUDE_API_KEY found");
            println!("   Using mock provider for demonstration");
            println!("\n   To use Claude API:");
            println!("   export ANTHROPIC_API_KEY=sk-ant-api03-...");
            println!("   or");
            println!("   export CLAUDE_API_KEY=sk-ant-api03-...\n");
            Arc::new(MockProvider::new())
        }
    };
    
    // Create fetcher
    let fetcher = Arc::new(Fetcher::new());
    
    // Configure for Claude's strengths
    let config = LLMExtractorConfig {
        format: ContentFormat::Markdown, // Claude excels with structured Markdown
        clean_html: true,
        max_content_length: 150_000, // Claude can handle very large contexts
        ..Default::default()
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    
    // Test comprehensive extraction
    println!("🔍 Comprehensive Web Page Analysis");
    println!("{}", "-".repeat(60));
    
    let test_urls = vec![
        ("https://docs.rust-lang.org/book/", "Rust Book Documentation"),
        ("https://www.tensorflow.org/", "TensorFlow Homepage"),
        ("https://nextjs.org/", "Next.js Framework"),
    ];
    
    for (url, description) in test_urls {
        println!("\n📄 Analyzing: {} - {}", url, description);
        println!("{}", "-".repeat(40));
        
        match extractor.extract::<WebPageAnalysis>(url, &fetcher).await {
            Ok(analysis) => {
                println!("✅ Analysis complete:");
                println!("   Title: {}", analysis.data.title);
                println!("   Type: {}", analysis.data.page_type);
                println!("   Purpose: {}", analysis.data.purpose);
                
                if let Some(cta) = &analysis.data.primary_cta {
                    println!("   Primary CTA: {}", cta);
                }
                
                println!("\n   📑 Sections found: {}", analysis.data.sections.len());
                for section in analysis.data.sections.iter().take(3) {
                    println!("      • {} ({})", section.title, section.importance);
                }
                
                println!("\n   💻 Technical Info:");
                if !analysis.data.technical_info.languages.is_empty() {
                    println!("      Languages: {}", analysis.data.technical_info.languages.join(", "));
                }
                if !analysis.data.technical_info.frameworks.is_empty() {
                    println!("      Frameworks: {}", analysis.data.technical_info.frameworks.join(", "));
                }
                println!("      Code Examples: {}", if analysis.data.technical_info.has_code_examples { "Yes" } else { "No" });
                println!("      API Docs: {}", if analysis.data.technical_info.has_api_docs { "Yes" } else { "No" });
                
                println!("\n   🔍 SEO Data:");
                println!("      Content Quality: {}/10", analysis.data.seo_data.content_quality_score);
                if !analysis.data.seo_data.keywords.is_empty() {
                    println!("      Keywords: {}", analysis.data.seo_data.keywords.join(", "));
                }
                
                if let Some(usage) = analysis.usage {
                    println!("\n   📊 Token Usage: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
                }
            }
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    
    // Show Claude API configuration options
    println!("\n\n🔧 Claude API Integration Options:");
    println!("{}", "=".repeat(60));
    println!("1. **Direct Anthropic SDK** (Recommended):");
    println!("   - Use the official anthropic-sdk-rust when available");
    println!("   - Direct access to all Claude features");
    println!("   - Best performance and reliability\n");
    
    println!("2. **OpenAI Compatibility Layer**:");
    println!("   - Some services provide Claude through OpenAI-compatible APIs");
    println!("   - Examples: OpenRouter, Helicone with Anthropic");
    println!("   - Configure base URL and headers accordingly\n");
    
    println!("3. **Custom Implementation**:");
    println!("   - Implement the LLMProvider trait for Anthropic");
    println!("   - Use reqwest to call Anthropic's REST API directly");
    println!("   - Full control over request/response handling\n");
    
    println!("📚 Example configuration for OpenRouter (Claude via OpenAI format):");
    println!("   ```rust");
    println!("   let config = OpenAIConfig::new()");
    println!("       .with_api_key(\"your-openrouter-key\")");
    println!("       .with_api_base(\"https://openrouter.ai/api/v1\")");
    println!("       .with_default_headers(headers);");
    println!("   ```");
    
    println!("\n🎉 Demo completed!");
    
    Ok(())
}