//! 直接使用 Anthropic API 进行结构化数据提取测试
//! 
//! 需要设置环境变量: ANTHROPIC_API_KEY
//! 运行: ANTHROPIC_API_KEY=your-key cargo run --example test_anthropic_direct --features llm

use url_preview::{
    LLMExtractor, AnthropicProvider, Fetcher, 
    PreviewError, ContentFormat, LLMExtractorConfig
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    main_topics: Vec<String>,
    key_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ProductInfo {
    name: String,
    description: String,
    price: Option<String>,
    features: Vec<String>,
    availability: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🚀 Anthropic API 结构化数据提取测试\n");
    
    // 检查 API Key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("请设置 ANTHROPIC_API_KEY 环境变量");
    
    // 创建 Anthropic Provider
    let provider = Arc::new(AnthropicProvider::new(api_key));
    
    // 配置提取器
    let config = LLMExtractorConfig {
        format: ContentFormat::Text, // 使用纯文本格式
        clean_html: true,
        max_content_length: 10_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    let fetcher = Fetcher::new();
    
    // 测试 1: 提取文章信息
    println!("📄 测试 1: 提取 Rust 官网文章信息");
    println!("{}", "=".repeat(50));
    
    match extractor.extract::<ArticleInfo>("https://www.rust-lang.org/", &fetcher).await {
        Ok(result) => {
            println!("✅ 成功提取！\n");
            println!("标题: {}", result.data.title);
            println!("摘要: {}", result.data.summary);
            
            println!("\n主要主题:");
            for topic in &result.data.main_topics {
                println!("  • {}", topic);
            }
            
            println!("\n关键点:");
            for point in &result.data.key_points {
                println!("  • {}", point);
            }
            
            if let Some(usage) = result.usage {
                println!("\nToken 使用: {} + {} = {}", 
                    usage.prompt_tokens, 
                    usage.completion_tokens, 
                    usage.total_tokens
                );
            }
        }
        Err(e) => println!("❌ 错误: {}", e),
    }
    
    // 测试 2: 提取 GitHub 项目信息
    println!("\n\n🔧 测试 2: 提取 GitHub 项目信息");
    println!("{}", "=".repeat(50));
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct GitHubProject {
        name: String,
        description: String,
        language: String,
        stars: Option<String>,
        last_updated: Option<String>,
        main_features: Vec<String>,
    }
    
    match extractor.extract::<GitHubProject>("https://github.com/rust-lang/rust", &fetcher).await {
        Ok(result) => {
            println!("✅ 成功提取！\n");
            println!("项目名: {}", result.data.name);
            println!("描述: {}", result.data.description);
            println!("语言: {}", result.data.language);
            
            if let Some(stars) = result.data.stars {
                println!("Stars: {}", stars);
            }
            
            println!("\n主要特性:");
            for feature in &result.data.main_features {
                println!("  • {}", feature);
            }
        }
        Err(e) => println!("❌ 错误: {}", e),
    }
    
    Ok(())
}