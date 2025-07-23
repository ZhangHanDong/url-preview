//! 使用 OpenAI API 进行结构化数据提取测试
//! 
//! 需要设置环境变量: OPENAI_API_KEY
//! 运行: OPENAI_API_KEY=your-key cargo run --example test_openai_extraction --features llm

use url_preview::{
    LLMExtractor, OpenAIProvider, Fetcher, 
    PreviewError, ContentFormat, LLMExtractorConfig
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebPageAnalysis {
    title: String,
    description: String,
    content_type: String, // article, product, documentation, etc.
    main_topics: Vec<String>,
    target_audience: String,
    key_information: Vec<KeyInfo>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct KeyInfo {
    category: String,
    details: String,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🚀 OpenAI API 结构化数据提取测试\n");
    
    // 检查 API Key
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("请设置 OPENAI_API_KEY 环境变量");
    
    // 创建 OpenAI Provider
    let provider = Arc::new(OpenAIProvider::new(api_key));
    
    // 配置提取器
    let config = LLMExtractorConfig {
        format: ContentFormat::Markdown, // OpenAI 处理 Markdown 效果好
        clean_html: true,
        max_content_length: 15_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    let fetcher = Fetcher::new();
    
    // 测试不同类型的网页
    let test_urls = vec![
        ("https://www.rust-lang.org/", "Rust 官网"),
        ("https://github.com/tokio-rs/tokio", "Tokio 项目"),
        ("https://docs.rs/", "docs.rs 文档站"),
    ];
    
    for (url, name) in test_urls {
        println!("\n📄 测试: {}", name);
        println!("URL: {}", url);
        println!("{}", "=".repeat(50));
        
        match extractor.extract::<WebPageAnalysis>(url, &fetcher).await {
            Ok(result) => {
                println!("✅ 成功提取！\n");
                println!("标题: {}", result.data.title);
                println!("描述: {}", result.data.description);
                println!("内容类型: {}", result.data.content_type);
                println!("目标受众: {}", result.data.target_audience);
                
                println!("\n主要主题:");
                for topic in &result.data.main_topics {
                    println!("  • {}", topic);
                }
                
                println!("\n关键信息:");
                for info in &result.data.key_information {
                    println!("  • [{}] {}", info.category, info.details);
                }
                
                if let Some(usage) = result.usage {
                    println!("\nToken 使用: {}", usage.total_tokens);
                }
            }
            Err(e) => println!("❌ 错误: {}", e),
        }
        
        println!();
    }
    
    // 测试复杂的电商产品页面
    println!("\n🛍️ 测试电商产品提取");
    println!("{}", "=".repeat(50));
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct EcommerceProduct {
        product_name: String,
        brand: Option<String>,
        price: Option<String>,
        currency: Option<String>,
        availability: String,
        rating: Option<f32>,
        review_count: Option<i32>,
        description: String,
        features: Vec<String>,
        specifications: Vec<Specification>,
    }
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct Specification {
        name: String,
        value: String,
    }
    
    // 这里可以测试一个真实的电商网站
    // let product_url = "https://example-shop.com/product";
    // match extractor.extract::<EcommerceProduct>(product_url, &fetcher).await {
    //     Ok(result) => { ... }
    //     Err(e) => { ... }
    // }
    
    println!("\n✨ 测试完成！");
    println!("\n💡 提示:");
    println!("1. OpenAI 的 function calling 功能非常适合结构化提取");
    println!("2. 使用 gpt-4 模型可以获得更好的提取质量");
    println!("3. 可以通过调整 schema 来控制提取的详细程度");
    
    Ok(())
}