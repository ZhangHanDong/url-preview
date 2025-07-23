//! Claude Code SDK provider implementation
//!
//! This module provides integration with the Claude Code CLI through the rust-claude-code-api

use super::*;
use cc_sdk::{query, ClaudeCodeOptions, Message, ContentBlock, Result as ClaudeResult};
use futures::StreamExt;

/// Claude Code SDK provider implementation
pub struct ClaudeCodeProvider {
    system_prompt: String,
    model: String,
    max_thinking_tokens: i32,
}

impl ClaudeCodeProvider {
    /// Create a new Claude Code provider
    pub fn new() -> Self {
        Self {
            system_prompt: "You are an expert at extracting structured data from web content. \
                           Always respond with valid JSON that matches the requested schema."
                .to_string(),
            model: "claude-3-opus-20240229".to_string(),
            max_thinking_tokens: 5000,
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
    
    /// Set custom system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }
    
    /// Set max thinking tokens
    pub fn with_max_thinking_tokens(mut self, tokens: i32) -> Self {
        self.max_thinking_tokens = tokens;
        self
    }
}

impl Default for ClaudeCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "claude-code"
    }
    
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError> {
        // Build the extraction prompt
        let full_prompt = format!(
            "Extract structured data from the following web content.\n\n\
            Expected data structure (JSON Schema):\n{}\n\n\
            Content to analyze:\n{}\n\n\
            Return ONLY valid JSON matching the schema. Do not include any explanation or markdown formatting.",
            serde_json::to_string_pretty(&schema)
                .map_err(|e| PreviewError::ParseError(e.to_string()))?,
            prompt
        );
        
        // Configure Claude options
        let options = ClaudeCodeOptions::builder()
            .system_prompt(&self.system_prompt)
            .model(&self.model)
            .max_thinking_tokens(self.max_thinking_tokens)
            .build();
        
        // Query Claude
        let mut messages = query(full_prompt, Some(options))
            .await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "Claude Code".to_string(),
                message: format!("Failed to query Claude: {:?}", e),
            })?;
        
        let mut json_response = String::new();
        let mut found_json = false;
        
        // Collect the response
        while let Some(msg) = messages.next().await {
            match msg.map_err(|e| PreviewError::ExternalServiceError {
                service: "Claude Code".to_string(),
                message: format!("Stream error: {}", e),
            })? {
                Message::Assistant { message } => {
                    for block in &message.content {
                        if let ContentBlock::Text(text) = block {
                            // Try to extract JSON from the response
                            let content = &text.text;
                            
                            // Look for JSON in the content
                            if let Some(start) = content.find('{') {
                                if let Some(end) = content.rfind('}') {
                                    json_response = content[start..=end].to_string();
                                    found_json = true;
                                } else {
                                    json_response.push_str(&content[start..]);
                                    found_json = true;
                                }
                            } else if found_json {
                                // Continue building JSON if we're in the middle of it
                                if let Some(end) = content.rfind('}') {
                                    json_response.push_str(&content[..=end]);
                                } else {
                                    json_response.push_str(content);
                                }
                            }
                        }
                    }
                }
                Message::Result { .. } => break,
                _ => {}
            }
        }
        
        // Parse the JSON response
        if json_response.is_empty() {
            return Err(PreviewError::ExtractError(
                "No JSON response from Claude".to_string()
            ));
        }
        
        serde_json::from_str(&json_response)
            .map_err(|e| PreviewError::ParseError(
                format!("Failed to parse Claude's response as JSON: {}\nResponse: {}", e, json_response)
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() {
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.name(), "claude-code");
        assert_eq!(provider.model, "claude-3-opus-20240229");
        
        let provider = ClaudeCodeProvider::new().with_sonnet();
        assert_eq!(provider.model, "claude-3-sonnet-20240229");
        
        let provider = ClaudeCodeProvider::new().with_haiku();
        assert_eq!(provider.model, "claude-3-haiku-20240307");
    }
}