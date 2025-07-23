//! Full integration test of claude-code-sdk with url-preview
//! 
//! This example shows how to use the claude-code-sdk as an LLM provider
//! for extracting structured data from web pages using url-preview.
//!
//! Prerequisites:
//! 1. Install Claude Code CLI: npm install -g @anthropic-ai/claude-code
//! 2. Build with llm feature: cargo build --features llm
//! 3. Run with: cargo run --example test_claude_code_integration --features llm

use url_preview::{
    PreviewService, Fetcher, FetchResult, 
    LLMExtractor, LLMExtractorConfig, 
    ClaudeCodeProvider, LLMProvider,
    PreviewError, ContentFormat,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Define various data structures we want to extract
#[derive(Debug, Serialize, Deserialize)]
struct WebPageSummary {
    title: String,
    description: String,
    main_topics: Vec<String>,
    key_points: Vec<String>,
    author_info: Option<AuthorInfo>,
    publish_date: Option<String>,
    reading_time_minutes: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthorInfo {
    name: String,
    bio: Option<String>,
    social_links: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductInfo {
    name: String,
    description: String,
    price: Option<PriceInfo>,
    features: Vec<String>,
    specifications: Option<Vec<Specification>>,
    availability: String,
    rating: Option<RatingInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PriceInfo {
    amount: f64,
    currency: String,
    discount_percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Specification {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RatingInfo {
    average: f32,
    count: u32,
    distribution: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TechnicalDocumentation {
    title: String,
    overview: String,
    sections: Vec<DocSection>,
    code_examples: Vec<CodeExample>,
    related_links: Vec<RelatedLink>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocSection {
    heading: String,
    content: String,
    subsections: Option<Vec<DocSection>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeExample {
    language: String,
    description: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RelatedLink {
    title: String,
    url: String,
    category: String,
}

async fn test_basic_extraction(extractor: &LLMExtractor) -> Result<(), PreviewError> {
    println!("\n🔍 Test 1: Basic Web Page Summary");
    println!("{}", "=".repeat(60));
    
    let url = "https://www.rust-lang.org/";
    println!("URL: {}", url);
    
    let result = extractor.extract::<WebPageSummary>(url).await?;
    
    println!("\n📊 Extraction Results:");
    println!("Title: {}", result.data.title);
    println!("Description: {}", result.data.description);
    println!("\nMain Topics:");
    for topic in &result.data.main_topics {
        println!("  • {}", topic);
    }
    println!("\nKey Points:");
    for (i, point) in result.data.key_points.iter().enumerate() {
        println!("  {}. {}", i + 1, point);
    }
    
    if let Some(author) = &result.data.author_info {
        println!("\nAuthor: {}", author.name);
        if let Some(bio) = &author.bio {
            println!("Bio: {}", bio);
        }
    }
    
    if let Some(usage) = &result.token_usage {
        println!("\n📈 Token Usage:");
        println!("  Input: {} tokens", usage.input_tokens);
        println!("  Output: {} tokens", usage.output_tokens);
        println!("  Total: {} tokens", usage.total_tokens);
    }
    
    Ok(())
}

async fn test_product_extraction(extractor: &LLMExtractor) -> Result<(), PreviewError> {
    println!("\n\n🛍️ Test 2: Product Information Extraction");
    println!("{}", "=".repeat(60));
    
    let url = "https://www.rust-lang.org/tools/install";
    println!("URL: {}", url);
    
    let result = extractor.extract::<ProductInfo>(url).await?;
    
    println!("\n📦 Product Details:");
    println!("Name: {}", result.data.name);
    println!("Description: {}", result.data.description);
    println!("Availability: {}", result.data.availability);
    
    println!("\nFeatures:");
    for feature in &result.data.features {
        println!("  ✓ {}", feature);
    }
    
    if let Some(specs) = &result.data.specifications {
        println!("\nSpecifications:");
        for spec in specs {
            println!("  • {}: {}", spec.name, spec.value);
        }
    }
    
    Ok(())
}

async fn test_documentation_extraction(extractor: &LLMExtractor) -> Result<(), PreviewError> {
    println!("\n\n📚 Test 3: Technical Documentation Extraction");
    println!("{}", "=".repeat(60));
    
    let url = "https://doc.rust-lang.org/book/";
    println!("URL: {}", url);
    
    let result = extractor.extract::<TechnicalDocumentation>(url).await?;
    
    println!("\n📖 Documentation Structure:");
    println!("Title: {}", result.data.title);
    println!("Overview: {}", result.data.overview);
    
    println!("\nSections:");
    for section in &result.data.sections[..5.min(result.data.sections.len())] {
        println!("  📑 {}", section.heading);
        if let Some(first_line) = section.content.lines().next() {
            println!("     {}", first_line);
        }
    }
    
    if !result.data.code_examples.is_empty() {
        println!("\nCode Examples:");
        for (i, example) in result.data.code_examples.iter().take(3).enumerate() {
            println!("  {}. {} ({})", i + 1, example.description, example.language);
        }
    }
    
    Ok(())
}

async fn test_with_custom_config() -> Result<(), PreviewError> {
    println!("\n\n⚙️ Test 4: Custom Configuration");
    println!("{}", "=".repeat(60));
    
    // Create custom provider with different model
    let provider = ClaudeCodeProvider::new()
        .with_sonnet() // Use faster Sonnet model
        .with_system_prompt(
            "You are an expert data analyst. Extract precise, structured data from web content. \
             Focus on accuracy and completeness."
        );
    
    // Create custom config
    let config = LLMExtractorConfig::builder()
        .max_content_length(10000)
        .include_images(true)
        .include_links(true)
        .content_format(ContentFormat::PlainText)
        .build();
    
    // Create extractor with custom provider and config
    let extractor = LLMExtractor::with_provider_and_config(
        Box::new(provider),
        config
    );
    
    // Test with a complex page
    let url = "https://github.com/rust-lang/rust";
    println!("Testing with custom config on: {}", url);
    
    #[derive(Debug, Serialize, Deserialize)]
    struct GitHubRepoInfo {
        name: String,
        description: String,
        stars: Option<u32>,
        language: String,
        topics: Vec<String>,
        license: Option<String>,
        last_commit: Option<String>,
    }
    
    let result = extractor.extract::<GitHubRepoInfo>(url).await?;
    
    println!("\n🔧 Repository Info:");
    println!("Name: {}", result.data.name);
    println!("Description: {}", result.data.description);
    println!("Language: {}", result.data.language);
    if let Some(stars) = result.data.stars {
        println!("Stars: ⭐ {}", stars);
    }
    println!("Topics: {}", result.data.topics.join(", "));
    
    Ok(())
}

async fn test_concurrent_extractions() -> Result<(), PreviewError> {
    println!("\n\n🚀 Test 5: Concurrent Extractions");
    println!("{}", "=".repeat(60));
    
    let provider = Arc::new(ClaudeCodeProvider::new().with_haiku()); // Use fastest model
    let extractor = Arc::new(LLMExtractor::with_provider(provider));
    
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://blog.rust-lang.org/",
        "https://crates.io/",
    ];
    
    println!("Extracting from {} URLs concurrently...", urls.len());
    
    let mut handles = vec![];
    
    for url in urls {
        let extractor = Arc::clone(&extractor);
        let url = url.to_string();
        
        let handle = tokio::spawn(async move {
            let result = extractor.extract::<WebPageSummary>(&url).await;
            (url, result)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        let (url, result) = handle.await.unwrap();
        match result {
            Ok(data) => {
                println!("\n✅ {}", url);
                println!("   Title: {}", data.data.title);
                println!("   Topics: {}", data.data.main_topics.join(", "));
            }
            Err(e) => {
                println!("\n❌ {} - Error: {}", url, e);
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 URL Preview + Claude Code SDK Integration Test");
    println!("{}", "=".repeat(60));
    println!("This example demonstrates using claude-code-sdk as an LLM provider");
    println!("for extracting structured data from web pages.\n");
    
    // Create the Claude Code provider
    let provider = ClaudeCodeProvider::new();
    println!("Provider: {}", provider.name());
    
    // Create LLM extractor with default config
    let extractor = LLMExtractor::with_provider(Box::new(provider));
    
    // Run tests
    if let Err(e) = test_basic_extraction(&extractor).await {
        eprintln!("Basic extraction error: {}", e);
    }
    
    if let Err(e) = test_product_extraction(&extractor).await {
        eprintln!("Product extraction error: {}", e);
    }
    
    if let Err(e) = test_documentation_extraction(&extractor).await {
        eprintln!("Documentation extraction error: {}", e);
    }
    
    if let Err(e) = test_with_custom_config().await {
        eprintln!("Custom config error: {}", e);
    }
    
    if let Err(e) = test_concurrent_extractions().await {
        eprintln!("Concurrent extraction error: {}", e);
    }
    
    println!("\n\n🎉 All tests completed!");
    println!("\n💡 Next Steps:");
    println!("1. Install Claude Code CLI if not already installed:");
    println!("   npm install -g @anthropic-ai/claude-code");
    println!("\n2. Use ClaudeCodeProvider in your own code:");
    println!("   let provider = ClaudeCodeProvider::new();");
    println!("   let extractor = LLMExtractor::with_provider(Box::new(provider));");
    println!("\n3. Extract custom data structures:");
    println!("   let result = extractor.extract::<YourStruct>(url).await?;");
    
    Ok(())
}