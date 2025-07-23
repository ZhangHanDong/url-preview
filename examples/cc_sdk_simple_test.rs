//! 最简单的 cc-sdk 测试 - 使用默认设置
//! 
//! cargo run --example cc_sdk_simple_test --features claude-code

use cc_sdk::{query, ClaudeCodeOptions, Message, ContentBlock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试 cc-sdk (默认配置)...\n");
    
    // 1. 最简单的调用 - 不设置任何选项
    let prompt = "Say 'Hello from cc-sdk!' and nothing else.";
    
    println!("测试 1: 最简单调用");
    match query(prompt, None).await {
        Ok(mut stream) => {
            println!("✅ 成功!");
            while let Some(msg) = futures::StreamExt::next(&mut stream).await {
                match msg {
                    Ok(Message::Assistant { message }) => {
                        for block in &message.content {
                            if let ContentBlock::Text(text) = block {
                                print!("{}", text.text);
                            }
                        }
                    }
                    Ok(Message::Result { .. }) => break,
                    _ => {}
                }
            }
            println!();
        }
        Err(e) => {
            println!("❌ 错误: {:?}", e);
        }
    }
    
    // 2. 测试 JSON 提取 - 只设置系统提示
    println!("\n测试 2: JSON 提取 (仅系统提示)");
    let options = ClaudeCodeOptions::builder()
        .system_prompt("Return only valid JSON.")
        .build();
    
    let json_prompt = r#"Return this JSON: {"status": "ok", "message": "cc-sdk works!"}"#;
    
    match query(json_prompt, Some(options)).await {
        Ok(mut stream) => {
            println!("✅ 成功!");
            let mut response = String::new();
            while let Some(msg) = futures::StreamExt::next(&mut stream).await {
                match msg {
                    Ok(Message::Assistant { message }) => {
                        for block in &message.content {
                            if let ContentBlock::Text(text) = block {
                                response.push_str(&text.text);
                            }
                        }
                    }
                    Ok(Message::Result { .. }) => break,
                    _ => {}
                }
            }
            println!("响应: {}", response);
            
            // 尝试解析 JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                println!("✅ JSON 解析成功: {}", serde_json::to_string_pretty(&json)?);
            }
        }
        Err(e) => {
            println!("❌ 错误: {:?}", e);
        }
    }
    
    // 3. 测试与 url-preview 集成
    println!("\n测试 3: url-preview 集成");
    test_with_url_preview().await;
    
    Ok(())
}

async fn test_with_url_preview() {
    use url_preview::{ClaudeCodeProvider, LLMExtractor, Fetcher};
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    use std::sync::Arc;
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestInfo {
        content: String,
    }
    
    // 不指定模型，使用默认
    let provider = Arc::new(ClaudeCodeProvider::new());
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    match extractor.extract::<TestInfo>("https://example.com", &fetcher).await {
        Ok(result) => {
            println!("✅ 提取成功: {}", result.data.content);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }
}