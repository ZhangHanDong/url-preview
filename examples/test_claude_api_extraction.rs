//! Test LLM extraction using claude-code-api-rs as OpenAI-compatible endpoint
//! 
//! This example demonstrates how to use claude-code-api-rs to extract structured data
//! from web pages using the OpenAI-compatible API without managing API keys.
//!
//! Prerequisites:
//! 1. Install and run claude-code-api-rs:
//!    cargo install claude-code-api
//!    RUST_LOG=info claude-code-api
//! 
//! 2. Run this example:
//!    cargo run --example test_claude_api_extraction --features llm

use url_preview::{
    PreviewService, Fetcher, 
    LLMExtractor, LLMExtractorConfig, 
    OpenAIProvider, LLMProvider,
    PreviewError, ContentFormat,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

// Define structured data types for extraction

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct NewsArticle {
    title: String,
    author: Option<AuthorInfo>,
    publish_date: Option<String>,
    summary: String,
    key_points: Vec<String>,
    topics: Vec<String>,
    sentiment: String,
    reading_time_minutes: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct AuthorInfo {
    name: String,
    bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ProductListing {
    name: String,
    brand: Option<String>,
    price: Option<PriceInfo>,
    description: String,
    features: Vec<String>,
    specifications: Vec<Specification>,
    availability: String,
    rating: Option<RatingInfo>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PriceInfo {
    amount: f64,
    currency: String,
    discount_percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Specification {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct RatingInfo {
    average: f32,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct TechnicalDocumentation {
    title: String,
    version: Option<String>,
    overview: String,
    sections: Vec<DocSection>,
    code_examples: Vec<CodeExample>,
    api_references: Vec<ApiReference>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct DocSection {
    heading: String,
    level: u32,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CodeExample {
    language: String,
    title: String,
    code: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ApiReference {
    endpoint: String,
    method: String,
    description: String,
    parameters: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct GitHubProject {
    name: String,
    description: String,
    language: String,
    stars: Option<u32>,
    forks: Option<u32>,
    topics: Vec<String>,
    license: Option<String>,
    last_commit: Option<String>,
    contributors: Option<u32>,
    open_issues: Option<u32>,
}

// Create OpenAI provider configured for claude-code-api
fn create_claude_api_provider() -> Arc<dyn LLMProvider> {
    // Configure to use local claude-code-api server
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed"); // API key is not required
    
    // Create provider with claude model
    let provider = OpenAIProvider::from_config(config, "claude-opus-4-20250514".to_string());
    
    Arc::new(provider)
}

async fn test_news_article_extraction(extractor: &LLMExtractor, fetcher: &Fetcher) -> Result<(), PreviewError> {
    println!("\n📰 Test 1: News Article Extraction");
    println!("{}", "=".repeat(60));
    
    let url = "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0.html";
    println!("URL: {}", url);
    
    let result = extractor.extract::<NewsArticle>(url, fetcher).await?;
    
    println!("\n📊 Extracted Article Data:");
    println!("Title: {}", result.data.title);
    if let Some(author) = &result.data.author {
        println!("Author: {}", author.name);
    }
    if let Some(date) = &result.data.publish_date {
        println!("Published: {}", date);
    }
    println!("\nSummary: {}", result.data.summary);
    
    println!("\nKey Points:");
    for (i, point) in result.data.key_points.iter().enumerate() {
        println!("  {}. {}", i + 1, point);
    }
    
    println!("\nTopics: {}", result.data.topics.join(", "));
    println!("Sentiment: {}", result.data.sentiment);
    
    if let Some(time) = result.data.reading_time_minutes {
        println!("Reading Time: {} minutes", time);
    }
    
    if let Some(usage) = &result.usage {
        println!("\n📈 Token Usage:");
        println!("  Prompt: {} tokens", usage.prompt_tokens);
        println!("  Completion: {} tokens", usage.completion_tokens);
        println!("  Total: {} tokens", usage.total_tokens);
    }
    
    Ok(())
}

async fn test_product_extraction(extractor: &LLMExtractor, fetcher: &Fetcher) -> Result<(), PreviewError> {
    println!("\n\n🛍️ Test 2: Product Information Extraction");
    println!("{}", "=".repeat(60));
    
    let url = "https://www.rust-lang.org/tools/install";
    println!("URL: {}", url);
    
    let result = extractor.extract::<ProductListing>(url, fetcher).await?;
    
    println!("\n📦 Product Details:");
    println!("Name: {}", result.data.name);
    if let Some(brand) = &result.data.brand {
        println!("Brand: {}", brand);
    }
    println!("Description: {}", result.data.description);
    println!("Availability: {}", result.data.availability);
    
    println!("\nFeatures:");
    for feature in &result.data.features {
        println!("  ✓ {}", feature);
    }
    
    if !result.data.specifications.is_empty() {
        println!("\nSpecifications:");
        for spec in &result.data.specifications {
            println!("  • {}: {}", spec.name, spec.value);
        }
    }
    
    Ok(())
}

async fn test_documentation_extraction(extractor: &LLMExtractor, fetcher: &Fetcher) -> Result<(), PreviewError> {
    println!("\n\n📚 Test 3: Technical Documentation Extraction");
    println!("{}", "=".repeat(60));
    
    let url = "https://doc.rust-lang.org/book/";
    println!("URL: {}", url);
    
    let result = extractor.extract::<TechnicalDocumentation>(url, fetcher).await?;
    
    println!("\n📖 Documentation Structure:");
    println!("Title: {}", result.data.title);
    if let Some(version) = &result.data.version {
        println!("Version: {}", version);
    }
    println!("Overview: {}", result.data.overview);
    
    println!("\nSections:");
    for section in &result.data.sections[..5.min(result.data.sections.len())] {
        println!("  {} {}", "  ".repeat(section.level as usize), section.heading);
    }
    
    if !result.data.code_examples.is_empty() {
        println!("\nCode Examples:");
        for (i, example) in result.data.code_examples.iter().take(3).enumerate() {
            println!("  {}. {} ({})", i + 1, example.title, example.language);
            if let Some(desc) = &example.description {
                println!("     {}", desc);
            }
        }
    }
    
    Ok(())
}

async fn test_github_project_extraction(extractor: &LLMExtractor, fetcher: &Fetcher) -> Result<(), PreviewError> {
    println!("\n\n🐙 Test 4: GitHub Project Information");
    println!("{}", "=".repeat(60));
    
    let url = "https://github.com/rust-lang/rust";
    println!("URL: {}", url);
    
    let result = extractor.extract::<GitHubProject>(url, fetcher).await?;
    
    println!("\n🔧 Repository Information:");
    println!("Name: {}", result.data.name);
    println!("Description: {}", result.data.description);
    println!("Primary Language: {}", result.data.language);
    
    if let Some(stars) = result.data.stars {
        println!("Stars: ⭐ {}", stars);
    }
    if let Some(forks) = result.data.forks {
        println!("Forks: 🍴 {}", forks);
    }
    if let Some(issues) = result.data.open_issues {
        println!("Open Issues: 🐛 {}", issues);
    }
    
    if !result.data.topics.is_empty() {
        println!("\nTopics: {}", result.data.topics.join(", "));
    }
    
    if let Some(license) = &result.data.license {
        println!("License: {}", license);
    }
    
    Ok(())
}

async fn test_concurrent_extraction() -> Result<(), PreviewError> {
    println!("\n\n🚀 Test 5: Concurrent Extraction Performance");
    println!("{}", "=".repeat(60));
    
    let provider = create_claude_api_provider();
    let fetcher = Arc::new(Fetcher::new());
    let extractor = Arc::new(LLMExtractor::new(provider));
    
    let urls = vec![
        ("https://www.rust-lang.org/", "Rust Homepage"),
        ("https://blog.rust-lang.org/", "Rust Blog"),
        ("https://crates.io/", "Crates.io"),
    ];
    
    println!("Extracting from {} URLs concurrently...", urls.len());
    let start = tokio::time::Instant::now();
    
    let mut handles = vec![];
    
    for (url, name) in urls {
        let extractor = Arc::clone(&extractor);
        let fetcher = Arc::clone(&fetcher);
        let url = url.to_string();
        let name = name.to_string();
        
        let handle = tokio::spawn(async move {
            let result = extractor.extract::<NewsArticle>(&url, &*fetcher).await;
            (name, url, result)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        match handle.await {
            Ok((name, url, result)) => {
                match result {
                    Ok(data) => {
                        println!("\n✅ {} ({})", name, url);
                        println!("   Title: {}", data.data.title);
                        println!("   Topics: {}", data.data.topics.join(", "));
                    }
                    Err(e) => {
                        println!("\n❌ {} ({}) - Error: {}", name, url, e);
                    }
                }
            }
            Err(e) => {
                println!("\n❌ Task error: {}", e);
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("\nTotal time: {:.2}s", elapsed.as_secs_f64());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 URL Preview + Claude Code API Integration Test");
    println!("{}", "=".repeat(60));
    println!("This example uses claude-code-api-rs as an OpenAI-compatible");
    println!("endpoint for structured data extraction.\n");
    
    // Check if claude-code-api is running
    println!("⚠️  Make sure claude-code-api is running:");
    println!("   cargo install claude-code-api");
    println!("   RUST_LOG=info claude-code-api\n");
    
    // Create provider and extractor
    let provider = create_claude_api_provider();
    let fetcher = Fetcher::new();
    
    // Configure extractor
    let config = LLMExtractorConfig {
        format: ContentFormat::Html,
        clean_html: true,
        max_content_length: 50_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    
    // Quick connectivity test
    println!("Testing connection to claude-code-api...");
    match reqwest::get("http://localhost:8080/health").await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Successfully connected to claude-code-api!\n");
        }
        Ok(resp) => {
            println!("⚠️  claude-code-api returned status: {}", resp.status());
            println!("   Please check if the service is running correctly.\n");
        }
        Err(e) => {
            println!("❌ Failed to connect to claude-code-api: {}", e);
            println!("   Please start the service with: RUST_LOG=info claude-code-api");
            return Err(e.into());
        }
    }
    
    // Run tests
    if let Err(e) = test_news_article_extraction(&extractor, &fetcher).await {
        eprintln!("News article extraction error: {}", e);
    }
    
    if let Err(e) = test_product_extraction(&extractor, &fetcher).await {
        eprintln!("Product extraction error: {}", e);
    }
    
    if let Err(e) = test_documentation_extraction(&extractor, &fetcher).await {
        eprintln!("Documentation extraction error: {}", e);
    }
    
    if let Err(e) = test_github_project_extraction(&extractor, &fetcher).await {
        eprintln!("GitHub project extraction error: {}", e);
    }
    
    if let Err(e) = test_concurrent_extraction().await {
        eprintln!("Concurrent extraction error: {}", e);
    }
    
    println!("\n\n🎉 All tests completed!");
    
    println!("\n💡 Benefits of using claude-code-api:");
    println!("1. No API key management required");
    println!("2. Uses your local Claude CLI authentication");
    println!("3. OpenAI-compatible interface");
    println!("4. Connection pooling for better performance");
    println!("5. Built-in conversation management");
    println!("6. Support for multimodal inputs");
    
    Ok(())
}