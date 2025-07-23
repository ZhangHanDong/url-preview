//! 综合 LLM 功能测试
//! 
//! 这个示例展示了所有 LLM providers 的功能：
//! - OpenAI API
//! - Anthropic API  
//! - Claude-code-api
//! - Local/Ollama
//! - Mock provider
//!
//! 使用方法：
//! export OPENAI_API_KEY=your-key
//! export ANTHROPIC_API_KEY=your-key
//! cargo run --example comprehensive_llm_test --features llm

use url_preview::{
    LLMExtractor, LLMExtractorConfig, ContentFormat, Fetcher,
    OpenAIProvider, AnthropicProvider, LocalProvider, MockProvider,
    ApiKeyValidator, LLMConfig, PreviewError
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebPageInfo {
    title: String,
    description: String,
    main_topics: Vec<String>,
    content_type: String, // article, product, documentation, etc.
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 综合 LLM 功能测试");
    println!("{}", "=".repeat(60));
    
    // Test 1: Configuration and validation
    test_configuration().await?;
    
    // Test 2: Auto-detection
    test_auto_detection().await?;
    
    // Test 3: Provider-specific tests
    test_all_providers().await?;
    
    // Test 4: Content formats
    test_content_formats().await?;
    
    println!("\n✅ 所有测试完成！");
    Ok(())
}

async fn test_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 1. 配置验证测试");
    println!("{}", "-".repeat(30));
    
    // Test API key validation
    println!("  🔑 API Key 验证:");
    
    // Valid keys
    match ApiKeyValidator::validate_openai_key("sk-1234567890abcdefghij1234567890") {
        Ok(_) => println!("    ✅ OpenAI key validation: 通过"),
        Err(e) => println!("    ❌ OpenAI key validation: {}", e),
    }
    
    match ApiKeyValidator::validate_anthropic_key("sk-ant-1234567890abcdefghij1234567890") {
        Ok(_) => println!("    ✅ Anthropic key validation: 通过"),
        Err(e) => println!("    ❌ Anthropic key validation: {}", e),
    }
    
    // Invalid keys
    if ApiKeyValidator::validate_openai_key("invalid").is_err() {
        println!("    ✅ Invalid OpenAI key: 正确拒绝");
    }
    
    if ApiKeyValidator::validate_anthropic_key("invalid").is_err() {
        println!("    ✅ Invalid Anthropic key: 正确拒绝");
    }
    
    // Test model validation
    println!("  🤖 模型验证:");
    
    if ApiKeyValidator::validate_model_name("openai", "gpt-4o-mini").is_ok() {
        println!("    ✅ OpenAI model validation: 通过");
    }
    
    if ApiKeyValidator::validate_model_name("anthropic", "claude-3-5-sonnet-20241022").is_ok() {
        println!("    ✅ Anthropic model validation: 通过");
    }
    
    if ApiKeyValidator::validate_model_name("openai", "invalid-model").is_err() {
        println!("    ✅ Invalid model: 正确拒绝");
    }
    
    Ok(())
}

async fn test_auto_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 2. 自动检测测试");
    println!("{}", "-".repeat(30));
    
    let provider = LLMConfig::auto_detect_provider().await?;
    println!("  ✅ 自动检测到 provider: {}", provider.name());
    
    Ok(())
}

async fn test_all_providers() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 3. Provider 特定测试");
    println!("{}", "-".repeat(30));
    
    let fetcher = Fetcher::new();
    let test_url = "https://www.rust-lang.org/";
    
    // Test Mock Provider (always works)
    println!("  🎭 Mock Provider:");
    test_provider_extraction(
        Arc::new(MockProvider::new()),
        test_url,
        &fetcher,
        "Mock"
    ).await;
    
    // Test OpenAI (if API key available)
    if let Ok(provider) = LLMConfig::openai_from_env() {
        println!("  🤖 OpenAI Provider:");
        test_provider_extraction(
            Arc::new(provider),
            test_url,
            &fetcher,
            "OpenAI"
        ).await;
    } else {
        println!("  ⚠️  OpenAI Provider: OPENAI_API_KEY 未设置");
    }
    
    // Test Anthropic (if API key available)
    if let Ok(provider) = LLMConfig::anthropic_from_env() {
        println!("  🧠 Anthropic Provider:");
        test_provider_extraction(
            Arc::new(provider),
            test_url,
            &fetcher,
            "Anthropic"
        ).await;
    } else {
        println!("  ⚠️  Anthropic Provider: ANTHROPIC_API_KEY 未设置");
    }
    
    // Test Claude Code API (if accessible)
    if let Ok(provider) = LLMConfig::claude_code_from_env() {
        println!("  🔮 Claude Code API:");
        test_provider_extraction(
            Arc::new(provider),
            test_url,
            &fetcher,
            "Claude-Code-API"
        ).await;
    } else {
        println!("  ⚠️  Claude Code API: 不可访问");
    }
    
    // Test Local Provider (if accessible)
    if let Ok(provider) = LLMConfig::local_from_env() {
        println!("  🏠 Local Provider:");
        test_provider_extraction(
            Arc::new(provider),
            test_url,
            &fetcher,
            "Local"
        ).await;
    } else {
        println!("  ⚠️  Local Provider: LOCAL_LLM_ENDPOINT 未设置");
    }
    
    Ok(())
}

async fn test_provider_extraction(
    provider: Arc<dyn url_preview::LLMProvider>,
    url: &str,
    fetcher: &Fetcher,
    provider_name: &str
) {
    let config = LLMExtractorConfig {
        format: ContentFormat::Text,
        clean_html: true,
        max_content_length: 3_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    
    match extractor.extract::<WebPageInfo>(url, fetcher).await {
        Ok(result) => {
            println!("    ✅ {} 提取成功!", provider_name);
            println!("       标题: {}", result.data.title);
            println!("       类型: {}", result.data.content_type);
            println!("       主题数: {}", result.data.main_topics.len());
            if let Some(usage) = result.usage {
                println!("       Token 使用: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("    ❌ {} 提取失败: {}", provider_name, e);
        }
    }
}

async fn test_content_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 4. 内容格式测试");
    println!("{}", "-".repeat(30));
    
    let provider = Arc::new(MockProvider::new());
    let fetcher = Fetcher::new();
    let test_url = "https://www.rust-lang.org/";
    
    let formats = vec![
        (ContentFormat::Html, "HTML"),
        (ContentFormat::Markdown, "Markdown"),
        (ContentFormat::Text, "Plain Text"),
    ];
    
    for (format, name) in formats {
        println!("  📄 {} 格式:", name);
        
        let config = LLMExtractorConfig {
            format,
            clean_html: true,
            max_content_length: 5_000,
            model_params: Default::default(),
        };
        
        let extractor = LLMExtractor::with_config(provider.clone(), config);
        
        match extractor.extract::<WebPageInfo>(test_url, &fetcher).await {
            Ok(result) => {
                println!("    ✅ {} 格式处理成功", name);
                println!("       描述长度: {} 字符", result.data.description.len());
            }
            Err(e) => {
                println!("    ❌ {} 格式处理失败: {}", name, e);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_provider_works() {
        let provider = Arc::new(MockProvider::new());
        let fetcher = Fetcher::new();
        
        let extractor = LLMExtractor::new(provider);
        
        // This should work with mock data
        let result = extractor.extract::<WebPageInfo>("https://example.com", &fetcher).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_all_content_formats() {
        assert_eq!(ContentFormat::Html, ContentFormat::Html);
        assert_ne!(ContentFormat::Html, ContentFormat::Text);
        assert_ne!(ContentFormat::Markdown, ContentFormat::Text);
    }
}