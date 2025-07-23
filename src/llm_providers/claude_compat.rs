//! Claude API provider with OpenAI-compatible interface
//!
//! This module provides a way to use Claude's API through an OpenAI-compatible interface

use super::*;
use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};

/// Claude provider using OpenAI-compatible endpoint
pub struct ClaudeCompatProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl ClaudeCompatProvider {
    /// Create a new Claude-compatible provider
    pub fn new(api_key: String) -> Self {
        // Configure for Claude's OpenAI-compatible endpoint
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://api.anthropic.com/v1"); // Claude's base URL
        
        Self {
            client: Client::with_config(config),
            model: "claude-3-opus-20240229".to_string(), // Default to Opus
        }
    }
    
    /// Use Claude 3 Sonnet (faster and cheaper)
    pub fn with_sonnet(mut self) -> Self {
        self.model = "claude-3-sonnet-20240229".to_string();
        self
    }
    
    /// Use Claude 3 Haiku (fastest and cheapest)
    pub fn with_haiku(mut self) -> Self {
        self.model = "claude-3-haiku-20240307".to_string();
        self
    }
    
    /// Use a custom model
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl LLMProvider for ClaudeCompatProvider {
    fn name(&self) -> &str {
        "claude-compat"
    }
    
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        // Build the extraction prompt
        let system_prompt = format!(
            "You are a helpful assistant that extracts structured data from web content. \
             Extract information according to the following JSON schema:\n\n{}\n\n\
             Return only valid JSON that matches the schema.",
            serde_json::to_string_pretty(&schema)
                .map_err(|e| PreviewError::ParseError(e.to_string()))?
        );
        
        // Build messages
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .map_err(|e| PreviewError::ExternalServiceError {
                        service: "Claude".to_string(),
                        message: format!("Failed to build system message: {}", e),
                    })?
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!(
                        "Extract structured data from the following content:\n\n{}",
                        prompt
                    ))
                    .build()
                    .map_err(|e| PreviewError::ExternalServiceError {
                        service: "Claude".to_string(),
                        message: format!("Failed to build user message: {}", e),
                    })?
            ),
        ];
        
        // Create the request
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.1) // Low temperature for consistent extraction
            .max_tokens(4096u32)
            .build()
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "Claude".to_string(),
                message: format!("Failed to build request: {}", e),
            })?;
        
        // Send request
        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "Claude".to_string(),
                message: format!("API request failed: {}", e),
            })?;
        
        // Extract the response
        if let Some(choice) = response.choices.first() {
            if let Some(content) = &choice.message.content {
                // Parse the JSON response
                serde_json::from_str(content)
                    .map_err(|e| PreviewError::ParseError(
                        format!("Failed to parse Claude's response as JSON: {}", e)
                    ))
            } else {
                Err(PreviewError::ExtractError("No content in Claude's response".to_string()))
            }
        } else {
            Err(PreviewError::ExtractError("No choices in Claude's response".to_string()))
        }
    }
}