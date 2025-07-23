//! 自定义 Claude Provider 来处理纯文本响应
//! 
//! 这个示例展示如何创建一个自定义 provider 来解析 Claude 的文本响应
//! 运行: cargo run --example custom_claude_provider --features llm

use url_preview::{
    LLMExtractor, LLMProvider, LLMExtractorConfig, ExtractionResult,
    Fetcher, PreviewError, ContentFormat, TokenUsage
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

/// 自定义 Claude Provider，处理纯文本 JSON 响应
pub struct CustomClaudeProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl CustomClaudeProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            model,
        }
    }
    
    /// 从 Claude 的文本响应中提取 JSON
    fn extract_json_from_text(text: &str) -> Option<String> {
        // 查找 JSON 开始和结束位置
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        
        if start <= end {
            Some(text[start..=end].to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl LLMProvider for CustomClaudeProvider {
    fn name(&self) -> &str {
        "CustomClaude"
    }
    
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        // 构建请求，要求 Claude 返回 JSON
        let schema_str = serde_json::to_string_pretty(&schema)
            .map_err(|e| PreviewError::SerializationError(e.to_string()))?;
        
        let full_prompt = format!(
            "Extract the following information from the provided content and return ONLY valid JSON that matches this schema:\n\n{}\n\nContent:\n{}\n\nIMPORTANT: Return only the JSON object, no explanations or markdown.",
            schema_str,
            prompt
        );
        
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": full_prompt
            }],
            "max_tokens": 1000,
            "temperature": 0.0
        });
        
        // 发送请求
        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", "Bearer not-needed")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PreviewError::HttpError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(PreviewError::ExternalServiceError(
                "Claude".to_string(),
                format!("API error: {}", error_text)
            ));
        }
        
        let response_json: Value = response.json().await
            .map_err(|e| PreviewError::SerializationError(e.to_string()))?;
        
        // 从响应中提取内容
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| PreviewError::ExternalServiceError(
                "Claude".to_string(),
                "No content in response".to_string()
            ))?;
        
        // 尝试从文本中提取 JSON
        let json_str = Self::extract_json_from_text(content)
            .ok_or_else(|| PreviewError::ExternalServiceError(
                "Claude".to_string(),
                "No valid JSON found in response".to_string()
            ))?;
        
        // 解析 JSON
        serde_json::from_str(&json_str)
            .map_err(|e| PreviewError::SerializationError(
                format!("Failed to parse JSON from Claude response: {}", e)
            ))
    }
}

// 测试结构
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleData {
    title: String,
    summary: String,
    topics: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), PreviewError> {
    println!("🚀 Custom Claude Provider 测试\n");
    
    // 创建自定义 provider
    let provider = Arc::new(CustomClaudeProvider::new(
        "http://localhost:8080".to_string(),
        "claude-3-5-haiku-20241022".to_string(),
    ));
    
    // 创建提取器
    let config = LLMExtractorConfig {
        format: ContentFormat::Text,
        clean_html: true,
        max_content_length: 5_000,
        model_params: Default::default(),
    };
    
    let extractor = LLMExtractor::with_config(provider, config);
    let fetcher = Fetcher::new();
    
    // 测试提取
    println!("📄 测试自定义 Provider");
    println!("{}", "=".repeat(50));
    
    let url = "https://www.rust-lang.org/";
    println!("URL: {}", url);
    
    match extractor.extract::<ArticleData>(url, &fetcher).await {
        Ok(result) => {
            println!("\n✅ 提取成功！");
            println!("标题: {}", result.data.title);
            println!("摘要: {}", result.data.summary);
            println!("\n主题:");
            for topic in &result.data.topics {
                println!("  • {}", topic);
            }
        }
        Err(e) => {
            println!("\n❌ 错误: {}", e);
            println!("\n可能的原因:");
            println!("1. claude-code-api 未运行");
            println!("2. 502 错误表示 Claude CLI 未认证");
            println!("3. 响应格式不是预期的 JSON");
        }
    }
    
    Ok(())
}