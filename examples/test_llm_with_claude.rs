//! Test LLM extraction using Claude API with OpenAI-compatible endpoint
//!
//! Run with:
//! ```
//! OPENAI_API_KEY=your_claude_api_key cargo run --example test_llm_with_claude --features llm
//! ```

use url_preview::{
    Fetcher, LLMExtractor, LLMExtractorConfig, ContentFormat, 
    OpenAIProvider, LLMProvider,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::env;

// Define structured data types to extract
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleAnalysis {
    /// Main title of the article
    title: String,
    /// Core topic or subject
    topic: String,
    /// Key points or takeaways (3-5 items)
    key_points: Vec<String>,
    /// Overall sentiment (positive, negative, neutral)
    sentiment: String,
    /// Target audience
    audience: Option<String>,
    /// Estimated reading time in minutes
    reading_time: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ProductReview {
    /// Product name
    product_name: String,
    /// Overall rating (1-5)
    rating: f32,
    /// Pros/advantages
    pros: Vec<String>,
    /// Cons/disadvantages  
    cons: Vec<String>,
    /// Review summary
    summary: String,
    /// Would recommend (yes/no/maybe)
    recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CompanyProfile {
    /// Company name
    name: String,
    /// Industry/sector
    industry: String,
    /// Company mission or value proposition
    mission: String,
    /// Key products or services
    offerings: Vec<String>,
    /// Company size (startup, small, medium, large, enterprise)
    size: String,
    /// Notable achievements or differentiators
    achievements: Vec<String>,
}

/// Create an OpenAI-compatible provider configured for Claude
fn create_claude_provider(api_key: String) -> OpenAIProvider {
    // Claude's OpenAI-compatible endpoint
    let claude_endpoint = "https://api.anthropic.com/v1/messages";
    
    // Note: In a real implementation, you'd need to configure the base URL
    // For now, we'll use the standard OpenAI provider and note the configuration needed
    println!("📝 Note: To use Claude's API, you would need to:");
    println!("   1. Set the base URL to: {}", claude_endpoint);
    println!("   2. Use model: claude-3-opus-20240229 or claude-3-sonnet-20240229");
    println!("   3. Adjust the request format for Claude's message API\n");
    
    // For demonstration, we'll create a standard OpenAI provider
    // In production, you'd extend this to support custom endpoints
    OpenAIProvider::new(api_key)
        .with_model("gpt-4-turbo-preview".to_string()) // Would be claude-3-opus-20240229
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LLM Extraction with Claude API (OpenAI-compatible)");
    println!("{}", "=".repeat(60));
    
    // Get API key
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => {
            println!("✅ API key found");
            // Check if it looks like a Claude API key
            if key.starts_with("sk-ant-") {
                println!("🔍 Detected Claude API key format");
            }
            key
        }
        Err(_) => {
            eprintln!("❌ No OPENAI_API_KEY found!");
            eprintln!("   Set your Claude API key:");
            eprintln!("   export OPENAI_API_KEY=sk-ant-your-key-here");
            return Ok(());
        }
    };
    
    // Create provider
    let provider: Arc<dyn LLMProvider> = Arc::new(create_claude_provider(api_key));
    
    // Create fetcher
    let fetcher = Arc::new(Fetcher::new());
    
    // Configure extractor for optimal Claude performance
    let config = LLMExtractorConfig {
        format: ContentFormat::Markdown, // Claude works well with Markdown
        clean_html: true,
        max_content_length: 100_000, // Claude can handle large contexts
        ..Default::default()
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    
    // Test 1: Analyze a technical article
    println!("\n📰 Test 1: Technical Article Analysis");
    println!("{}", "-".repeat(60));
    
    let article_url = "https://www.rust-lang.org/what/networking";
    println!("URL: {}", article_url);
    
    match extractor.extract::<ArticleAnalysis>(article_url, &fetcher).await {
        Ok(analysis) => {
            println!("\n✅ Article Analysis:");
            println!("   Title: {}", analysis.data.title);
            println!("   Topic: {}", analysis.data.topic);
            println!("   Sentiment: {}", analysis.data.sentiment);
            println!("   Key Points:");
            for (i, point) in analysis.data.key_points.iter().enumerate() {
                println!("     {}. {}", i + 1, point);
            }
            if let Some(audience) = analysis.data.audience {
                println!("   Target Audience: {}", audience);
            }
            
            // Show token usage
            if let Some(usage) = analysis.usage {
                println!("\n   📊 Token Usage:");
                println!("      Prompt: {} tokens", usage.prompt_tokens);
                println!("      Completion: {} tokens", usage.completion_tokens);
                println!("      Total: {} tokens", usage.total_tokens);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 2: Extract company information
    println!("\n\n🏢 Test 2: Company Profile Extraction");
    println!("{}", "-".repeat(60));
    
    let company_url = "https://www.anthropic.com";
    println!("URL: {}", company_url);
    
    match extractor.extract::<CompanyProfile>(company_url, &fetcher).await {
        Ok(profile) => {
            println!("\n✅ Company Profile:");
            println!("   Name: {}", profile.data.name);
            println!("   Industry: {}", profile.data.industry);
            println!("   Mission: {}", profile.data.mission);
            println!("   Size: {}", profile.data.size);
            println!("   Offerings:");
            for offering in profile.data.offerings.iter().take(3) {
                println!("     • {}", offering);
            }
            if !profile.data.achievements.is_empty() {
                println!("   Key Achievements:");
                for achievement in profile.data.achievements.iter().take(3) {
                    println!("     ⭐ {}", achievement);
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Test 3: Complex extraction with nested data
    println!("\n\n📊 Test 3: Product Review Extraction");
    println!("{}", "-".repeat(60));
    
    // Note: This would work better with actual review pages
    let review_url = "https://www.rust-lang.org/tools/install";
    println!("URL: {} (simulated review)", review_url);
    
    match extractor.extract::<ProductReview>(review_url, &fetcher).await {
        Ok(review) => {
            println!("\n✅ Product Review:");
            println!("   Product: {}", review.data.product_name);
            println!("   Rating: {:.1}/5", review.data.rating);
            println!("   Summary: {}", review.data.summary);
            println!("   Recommendation: {}", review.data.recommendation);
            
            if !review.data.pros.is_empty() {
                println!("   Pros:");
                for pro in review.data.pros.iter() {
                    println!("     ✅ {}", pro);
                }
            }
            
            if !review.data.cons.is_empty() {
                println!("   Cons:");
                for con in review.data.cons.iter() {
                    println!("     ❌ {}", con);
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    // Tips for using Claude API
    println!("\n\n💡 Tips for using Claude API:");
    println!("{}", "=".repeat(60));
    println!("1. Claude excels at analysis and reasoning tasks");
    println!("2. Use clear, structured prompts for best results");
    println!("3. Claude can handle up to 200k tokens of context");
    println!("4. Consider using claude-3-sonnet for faster, cheaper extraction");
    println!("5. Claude is particularly good at:");
    println!("   • Summarization and key point extraction");
    println!("   • Sentiment analysis and tone detection");
    println!("   • Structured data extraction from unstructured text");
    println!("   • Multi-language content analysis");
    
    println!("\n🎉 Testing completed!");
    
    Ok(())
}