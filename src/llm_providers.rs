//! LLM provider implementations
//!
//! This module contains implementations for various LLM providers including
//! OpenAI, Anthropic, and others.

use crate::llm_extractor::{LLMProvider, LLMExtractorConfig};
use crate::PreviewError;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Mock LLM provider for testing
pub struct MockProvider {
    name: String,
    responses: HashMap<String, Value>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            responses: HashMap::new(),
        }
    }
    
    pub fn with_response(mut self, key: String, response: Value) -> Self {
        self.responses.insert(key, response);
        self
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn generate(
        &self,
        _prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        // For mock provider, return a simple response based on schema
        if let Some(properties) = schema.get("properties") {
            let mut result = serde_json::Map::new();
            
            if let Some(props) = properties.as_object() {
                for (key, prop_schema) in props {
                    let value = match prop_schema.get("type").and_then(|t| t.as_str()) {
                        Some("string") => Value::String("Mock value".to_string()),
                        Some("number") => Value::Number(serde_json::Number::from(42)),
                        Some("boolean") => Value::Bool(true),
                        Some("array") => Value::Array(vec![]),
                        Some("object") => Value::Object(serde_json::Map::new()),
                        _ => Value::Null,
                    };
                    result.insert(key.clone(), value);
                }
            }
            
            Ok(Value::Object(result))
        } else {
            Ok(Value::Object(serde_json::Map::new()))
        }
    }
}

#[cfg(feature = "async-openai")]
pub mod openai {
    use super::*;
    use async_openai::{Client, config::OpenAIConfig};
    use async_openai::types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        ChatCompletionToolArgs, ChatCompletionToolType,
        FunctionObjectArgs,
    };
    
    /// OpenAI provider implementation
    pub struct OpenAIProvider {
        client: Client<OpenAIConfig>,
        model: String,
    }
    
    impl OpenAIProvider {
        pub fn new(api_key: String) -> Self {
            let config = OpenAIConfig::new().with_api_key(api_key);
            Self {
                client: Client::with_config(config),
                model: "gpt-4-turbo-preview".to_string(),
            }
        }
        
        pub fn with_model(mut self, model: String) -> Self {
            self.model = model;
            self
        }
        
        /// Create from custom client configuration
        pub fn from_config(config: OpenAIConfig, model: String) -> Self {
            Self {
                client: Client::with_config(config),
                model,
            }
        }
        
        /// Extract JSON from text content
        fn extract_json_from_text(text: &str) -> Option<String> {
            // Find the first '{' and last '}'
            let start = text.find('{')?;
            let end = text.rfind('}')?;
            
            if start <= end {
                let potential_json = &text[start..=end];
                // Basic validation - check if it looks like valid JSON
                if potential_json.contains('"') || potential_json.contains(':') {
                    return Some(potential_json.to_string());
                }
            }
            
            None
        }
    }
    
    #[async_trait]
    impl LLMProvider for OpenAIProvider {
        fn name(&self) -> &str {
            "openai"
        }
        
        async fn generate(
            &self,
            prompt: String,
            schema: Value,
            _config: &LLMExtractorConfig,
        ) -> Result<Value, PreviewError> {
            // Build function definition
            let function = FunctionObjectArgs::default()
                .name("extract_data")
                .description("Extract structured data from the content")
                .parameters(schema)
                .build()
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            let tool = ChatCompletionToolArgs::default()
                .r#type(ChatCompletionToolType::Function)
                .function(function)
                .build()
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            // Build messages
            let system_message = ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant that extracts structured data from web content.")
                .build()
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            let user_message = ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            // Create request
            let request = CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(vec![
                    ChatCompletionRequestMessage::System(system_message),
                    ChatCompletionRequestMessage::User(user_message),
                ])
                .tools(vec![tool])
                .tool_choice("required")
                .build()
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            // Make API call
            let response = self.client
                .chat()
                .create(request)
                .await
                .map_err(|e| PreviewError::ExternalServiceError {
                    service: "OpenAI".to_string(),
                    message: e.to_string(),
                })?;
            
            // Extract function call arguments
            if let Some(choice) = response.choices.first() {
                // First try tool calls (OpenAI format)
                if let Some(tool_calls) = &choice.message.tool_calls {
                    if let Some(tool_call) = tool_calls.first() {
                        let args_str = &tool_call.function.arguments;
                        let args: Value = serde_json::from_str(args_str)
                            .map_err(|e| PreviewError::ParseError(e.to_string()))?;
                        return Ok(args);
                    }
                }
                
                // Fallback: Try to extract JSON from content (for Claude compatibility)
                if let Some(content) = &choice.message.content {
                    // First try to parse the entire content as JSON
                    if let Ok(json) = serde_json::from_str::<Value>(content) {
                        return Ok(json);
                    }
                    
                    // Try to extract JSON from text
                    if let Some(json_str) = Self::extract_json_from_text(content) {
                        if let Ok(json) = serde_json::from_str::<Value>(&json_str) {
                            return Ok(json);
                        }
                    }
                }
            }
            
            Err(PreviewError::ExternalServiceError {
                service: "OpenAI".to_string(),
                message: "No function call or valid JSON in response".to_string(),
            })
        }
    }
}

// Anthropic provider would go here when anthropic-sdk-rust is available
// #[cfg(feature = "anthropic")]
pub mod anthropic {
    use super::*;
    
    /// Anthropic Claude provider implementation
    pub struct AnthropicProvider {
        api_key: String,
        model: String,
    }
    
    impl AnthropicProvider {
        pub fn new(api_key: String) -> Self {
            Self {
                api_key,
                model: "claude-3-opus-20240229".to_string(),
            }
        }
        
        pub fn with_model(mut self, model: String) -> Self {
            self.model = model;
            self
        }
    }
    
    #[async_trait]
    impl LLMProvider for AnthropicProvider {
        fn name(&self) -> &str {
            "anthropic"
        }
        
        async fn generate(
            &self,
            prompt: String,
            schema: Value,
            _config: &LLMExtractorConfig,
        ) -> Result<Value, PreviewError> {
            // Build the system prompt with schema instructions
            let schema_str = serde_json::to_string_pretty(&schema)
                .map_err(|e| PreviewError::ParseError(e.to_string()))?;
            
            let system_prompt = format!(
                "You are a helpful assistant that extracts structured data from web content. \
                You must respond with valid JSON that exactly matches this schema:\n\n{}\n\n\
                Only return the JSON object, no explanations or markdown.",
                schema_str
            );
            
            // Build the request payload
            let request_body = serde_json::json!({
                "model": self.model,
                "max_tokens": 1000,
                "system": system_prompt,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            });
            
            // Make the API call
            let client = reqwest::Client::new();
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("anthropic-version", "2023-06-01")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| PreviewError::FetchError(e.to_string()))?;
            
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(PreviewError::ExternalServiceError {
                    service: "Anthropic".to_string(),
                    message: format!("API error: {}", error_text)
                });
            }
            
            let response_json: Value = response.json().await
                .map_err(|e| PreviewError::ParseError(e.to_string()))?;
            
            // Extract content from Claude's response
            let content = response_json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| PreviewError::ExternalServiceError {
                    service: "Anthropic".to_string(),
                    message: "No content in response".to_string()
                })?;
            
            // Try to parse the content as JSON
            match serde_json::from_str::<Value>(content) {
                Ok(json) => Ok(json),
                Err(_) => {
                    // Try to extract JSON from text
                    if let Some(json_str) = AnthropicProvider::extract_json_from_text(content) {
                        serde_json::from_str(&json_str)
                            .map_err(|e| PreviewError::ParseError(e.to_string()))
                    } else {
                        Err(PreviewError::ExternalServiceError {
                            service: "Anthropic".to_string(),
                            message: "Could not extract valid JSON from response".to_string()
                        })
                    }
                }
            }
        }
    }
    
    impl AnthropicProvider {
        /// Extract JSON from text content
        fn extract_json_from_text(text: &str) -> Option<String> {
            let start = text.find('{')?;
            let end = text.rfind('}')?;
            
            if start <= end {
                let potential_json = &text[start..=end];
                if potential_json.contains('"') || potential_json.contains(':') {
                    return Some(potential_json.to_string());
                }
            }
            
            None
        }
    }
}

/// Local/Ollama provider for running models locally
pub struct LocalProvider {
    endpoint: String,
    model: String,
}

impl LocalProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self { endpoint, model }
    }
}

// Claude-compatible provider module
pub mod claude_compat;

// Claude Code SDK provider module
#[cfg(feature = "cc-sdk")]
pub mod claude_code;

#[async_trait]
impl LLMProvider for LocalProvider {
    fn name(&self) -> &str {
        "local"
    }
    
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        // Build request for Ollama or local model server
        let schema_str = serde_json::to_string_pretty(&schema)
            .map_err(|e| PreviewError::ParseError(e.to_string()))?;
        
        let full_prompt = format!(
            "Extract structured data from the following content and return only valid JSON that matches this schema:\n\n{}\n\nContent:\n{}\n\nJSON:",
            schema_str,
            prompt
        );
        
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": full_prompt,
            "format": "json",
            "stream": false
        });
        
        // Make request to local model server (e.g., Ollama)
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| PreviewError::FetchError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PreviewError::ExternalServiceError {
                service: "Local".to_string(),
                message: format!("Local model server error: {}", response.status())
            });
        }
        
        let response_json: Value = response.json().await
            .map_err(|e| PreviewError::ParseError(e.to_string()))?;
        
        // Extract response from Ollama format
        let content = response_json["response"]
            .as_str()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "Local".to_string(),
                message: "No response field in local model output".to_string()
            })?;
        
        // Parse JSON response
        serde_json::from_str::<Value>(content)
            .or_else(|_| {
                // Try to extract JSON from text if direct parsing fails
                if let Some(json_str) = LocalProvider::extract_json_from_text(content) {
                    serde_json::from_str(&json_str)
                        .map_err(|e| PreviewError::ParseError(e.to_string()))
                } else {
                    Err(PreviewError::ExternalServiceError {
                        service: "Local".to_string(),
                        message: "Could not parse JSON from local model response".to_string()
                    })
                }
            })
    }
}

impl LocalProvider {
    /// Extract JSON from text content
    fn extract_json_from_text(text: &str) -> Option<String> {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        
        if start <= end {
            let potential_json = &text[start..=end];
            if potential_json.contains('"') || potential_json.contains(':') {
                return Some(potential_json.to_string());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new();
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "price": { "type": "number" },
                "available": { "type": "boolean" }
            }
        });
        
        let result = provider.generate(
            "Test prompt".to_string(),
            schema,
            &LLMExtractorConfig::default()
        ).await.unwrap();
        
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("title").unwrap().as_str().unwrap(), "Mock value");
        assert_eq!(obj.get("price").unwrap().as_i64().unwrap(), 42);
        assert_eq!(obj.get("available").unwrap().as_bool().unwrap(), true);
    }
}