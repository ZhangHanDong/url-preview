//! Practical web content extraction demo using claude-code-api
//!
//! This example shows real-world use cases for structured data extraction
//!
//! Prerequisites:
//! 1. Start claude-code-api: RUST_LOG=info claude-code-api
//! 2. Run: cargo run --example practical_extraction_demo --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

// 1. E-commerce Product Extraction
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EcommerceProduct {
    name: String,
    brand: Option<String>,
    price: Option<Price>,
    description: String,
    features: Vec<String>,
    technical_specs: Vec<TechSpec>,
    availability: String,
    images: Vec<String>,
    reviews_summary: Option<ReviewsSummary>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Price {
    amount: f64,
    currency: String,
    discount_percentage: Option<f64>,
    original_price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct TechSpec {
    category: String,
    specs: Vec<SpecItem>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SpecItem {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ReviewsSummary {
    average_rating: f32,
    total_reviews: u32,
    rating_distribution: Vec<RatingCount>,
    top_positive_points: Vec<String>,
    top_negative_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct RatingCount {
    stars: u8,
    count: u32,
}

// 2. News/Blog Article Analysis
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleAnalysis {
    metadata: ArticleMetadata,
    content_analysis: ContentAnalysis,
    seo_info: SeoInfo,
    engagement_metrics: Option<EngagementMetrics>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleMetadata {
    title: String,
    author: Option<Author>,
    publish_date: Option<String>,
    last_updated: Option<String>,
    categories: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Author {
    name: String,
    bio: Option<String>,
    social_links: Vec<SocialLink>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SocialLink {
    platform: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ContentAnalysis {
    summary: String,
    key_points: Vec<String>,
    sentiment: String,
    reading_time_minutes: u32,
    difficulty_level: String,
    main_topics: Vec<Topic>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Topic {
    name: String,
    relevance_score: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SeoInfo {
    meta_description: Option<String>,
    canonical_url: Option<String>,
    open_graph_data: Option<OpenGraphData>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct OpenGraphData {
    title: String,
    description: String,
    image: Option<String>,
    og_type: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EngagementMetrics {
    views: Option<u32>,
    likes: Option<u32>,
    comments: Option<u32>,
    shares: Option<u32>,
}

// 3. Company/Organization Profile
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CompanyProfile {
    basic_info: CompanyBasicInfo,
    business_details: BusinessDetails,
    contact_info: ContactInfo,
    social_presence: SocialPresence,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CompanyBasicInfo {
    name: String,
    tagline: Option<String>,
    founded_year: Option<u32>,
    headquarters: Option<String>,
    company_size: Option<String>,
    industry: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BusinessDetails {
    description: String,
    products_services: Vec<String>,
    key_differentiators: Vec<String>,
    target_market: Vec<String>,
    partnerships: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ContactInfo {
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    support_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SocialPresence {
    website: String,
    social_media: Vec<SocialAccount>,
    blog_url: Option<String>,
    career_page: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct SocialAccount {
    platform: String,
    url: String,
    follower_count: Option<u32>,
}

// Helper function to create Claude API client
fn create_extractor() -> LLMExtractor {
    let config = async_openai::config::OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed");
    
    let provider = Arc::new(
        OpenAIProvider::from_config(config, "claude-opus-4-20250514".to_string())
    );
    
    let extractor_config = LLMExtractorConfig {
        format: ContentFormat::Html,
        clean_html: true,
        max_content_length: 100_000, // Larger limit for detailed extraction
        model_params: Default::default(),
    };
    
    LLMExtractor::with_config(provider, extractor_config)
}

async fn extract_ecommerce_product(url: &str) -> Result<(), PreviewError> {
    println!("\n🛍️ E-commerce Product Extraction");
    println!("{}", "=".repeat(50));
    println!("URL: {}\n", url);
    
    let extractor = create_extractor();
    let fetcher = Fetcher::new();
    
    match extractor.extract::<EcommerceProduct>(url, &fetcher).await {
        Ok(result) => {
            let product = &result.data;
            println!("Product: {}", product.name);
            if let Some(brand) = &product.brand {
                println!("Brand: {}", brand);
            }
            
            if let Some(price) = &product.price {
                print!("Price: {:.2} {}", price.amount, price.currency);
                if let Some(discount) = price.discount_percentage {
                    print!(" (-{}%)", discount);
                }
                println!();
            }
            
            println!("\nDescription: {}", product.description);
            
            println!("\nKey Features:");
            for feature in &product.features[..5.min(product.features.len())] {
                println!("  • {}", feature);
            }
            
            if !product.technical_specs.is_empty() {
                println!("\nTechnical Specifications:");
                for category in &product.technical_specs {
                    println!("  {}:", category.category);
                    for spec in &category.specs[..3.min(category.specs.len())] {
                        println!("    - {}: {}", spec.name, spec.value);
                    }
                }
            }
            
            println!("\nAvailability: {}", product.availability);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

async fn analyze_article(url: &str) -> Result<(), PreviewError> {
    println!("\n📰 Article Analysis");
    println!("{}", "=".repeat(50));
    println!("URL: {}\n", url);
    
    let extractor = create_extractor();
    let fetcher = Fetcher::new();
    
    match extractor.extract::<ArticleAnalysis>(url, &fetcher).await {
        Ok(result) => {
            let article = &result.data;
            
            println!("Title: {}", article.metadata.title);
            if let Some(author) = &article.metadata.author {
                println!("Author: {}", author.name);
            }
            if let Some(date) = &article.metadata.publish_date {
                println!("Published: {}", date);
            }
            
            println!("\nSummary: {}", article.content_analysis.summary);
            
            println!("\nKey Points:");
            for point in &article.content_analysis.key_points {
                println!("  • {}", point);
            }
            
            println!("\nAnalysis:");
            println!("  Sentiment: {}", article.content_analysis.sentiment);
            println!("  Reading Time: {} minutes", article.content_analysis.reading_time_minutes);
            println!("  Difficulty: {}", article.content_analysis.difficulty_level);
            
            println!("\nMain Topics:");
            for topic in &article.content_analysis.main_topics[..3.min(article.content_analysis.main_topics.len())] {
                println!("  • {} (relevance: {:.1})", topic.name, topic.relevance_score);
            }
            
            if !article.metadata.tags.is_empty() {
                println!("\nTags: {}", article.metadata.tags.join(", "));
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

async fn extract_company_profile(url: &str) -> Result<(), PreviewError> {
    println!("\n🏢 Company Profile Extraction");
    println!("{}", "=".repeat(50));
    println!("URL: {}\n", url);
    
    let extractor = create_extractor();
    let fetcher = Fetcher::new();
    
    match extractor.extract::<CompanyProfile>(url, &fetcher).await {
        Ok(result) => {
            let company = &result.data;
            
            println!("Company: {}", company.basic_info.name);
            if let Some(tagline) = &company.basic_info.tagline {
                println!("Tagline: {}", tagline);
            }
            
            println!("\nAbout: {}", company.business_details.description);
            
            if !company.basic_info.industry.is_empty() {
                println!("\nIndustry: {}", company.basic_info.industry.join(", "));
            }
            
            println!("\nProducts/Services:");
            for item in &company.business_details.products_services[..5.min(company.business_details.products_services.len())] {
                println!("  • {}", item);
            }
            
            println!("\nKey Differentiators:");
            for diff in &company.business_details.key_differentiators {
                println!("  • {}", diff);
            }
            
            if !company.social_presence.social_media.is_empty() {
                println!("\nSocial Media:");
                for social in &company.social_presence.social_media {
                    print!("  • {}: {}", social.platform, social.url);
                    if let Some(followers) = social.follower_count {
                        print!(" ({} followers)", followers);
                    }
                    println!();
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Practical Web Content Extraction Demo");
    println!("{}", "=".repeat(60));
    println!("Using claude-code-api for structured data extraction\n");
    
    // Check if claude-code-api is running
    match reqwest::get("http://localhost:8080/health").await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Connected to claude-code-api\n");
        }
        _ => {
            println!("⚠️  Please start claude-code-api first:");
            println!("   RUST_LOG=info claude-code-api");
            return Ok(());
        }
    }
    
    // Demo URLs (you can replace with actual URLs)
    let demos = vec![
        ("https://www.rust-lang.org/tools/install", "Product/Tool Page"),
        ("https://blog.rust-lang.org/", "Blog/News Site"),
        ("https://www.mozilla.org/", "Company Website"),
    ];
    
    println!("Running extraction demos...\n");
    
    // Extract as product
    if let Err(e) = extract_ecommerce_product(demos[0].0).await {
        eprintln!("Product extraction error: {}", e);
    }
    
    // Analyze as article
    if let Err(e) = analyze_article(demos[1].0).await {
        eprintln!("Article analysis error: {}", e);
    }
    
    // Extract company profile
    if let Err(e) = extract_company_profile(demos[2].0).await {
        eprintln!("Company extraction error: {}", e);
    }
    
    println!("\n\n✅ Demo completed!");
    println!("\n💡 Use Cases:");
    println!("1. E-commerce: Product details, pricing, reviews");
    println!("2. Content: Article analysis, SEO data, engagement");
    println!("3. Business: Company profiles, contact info, social presence");
    println!("4. Research: Data collection, competitive analysis");
    println!("5. Monitoring: Price tracking, content changes");
    
    Ok(())
}