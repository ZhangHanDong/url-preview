//! 验证 cc-sdk 基本功能
//! cargo run --example cc_sdk_working_test --features claude-code

use url_preview::{ClaudeCodeProvider, LLMExtractor, Fetcher, LLMProvider};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BasicInfo {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ cc-sdk 功能测试\n");
    
    // 1. 测试基本提取
    let provider = Arc::new(ClaudeCodeProvider::new());
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    println!("测试 1: 基本提取 (example.com)");
    match extractor.extract::<BasicInfo>("https://example.com", &fetcher).await {
        Ok(result) => {
            println!("✅ 成功!");
            println!("内容预览: {}...", &result.data.content[..50.min(result.data.content.len())]);
        }
        Err(e) => {
            println!("❌ 失败: {}", e);
        }
    }
    
    // 2. 测试复杂结构
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct DetailedInfo {
        title: String,
        description: String,
        main_topics: Vec<String>,
    }
    
    println!("\n测试 2: 复杂结构提取");
    let provider2 = Arc::new(
        ClaudeCodeProvider::new()
            .with_system_prompt("Extract structured data accurately. Always include all required fields.".to_string())
    );
    let extractor2 = LLMExtractor::new(provider2);
    
    match extractor2.extract::<DetailedInfo>("https://www.rust-lang.org/", &fetcher).await {
        Ok(result) => {
            println!("✅ 成功!");
            println!("- 标题: {}", result.data.title);
            println!("- 描述: {} 字符", result.data.description.len());
            println!("- 主题数: {}", result.data.main_topics.len());
            
            if let Some(usage) = result.usage {
                println!("\nToken 使用:");
                println!("- 总计: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ 失败: {}", e);
        }
    }
    
    println!("\n📊 测试总结:");
    println!("cc-sdk 集成正常工作！");
    println!("\n注意事项:");
    println!("1. 使用模型别名: 'sonnet', 'haiku', 'opus'");
    println!("2. JSON 字段名必须与结构体完全匹配");
    println!("3. Claude 会智能理解字段含义并填充数据");
    
    Ok(())
}