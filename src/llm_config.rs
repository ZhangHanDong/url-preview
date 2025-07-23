//! LLM configuration and validation utilities

use crate::PreviewError;

/// API key validation utilities
pub struct ApiKeyValidator;

impl ApiKeyValidator {
    /// Validate OpenAI API key format
    pub fn validate_openai_key(api_key: &str) -> Result<(), PreviewError> {
        if api_key.is_empty() {
            return Err(PreviewError::InvalidConfiguration("OpenAI API key cannot be empty".to_string()));
        }
        
        if !api_key.starts_with("sk-") {
            return Err(PreviewError::InvalidConfiguration(
                "OpenAI API key must start with 'sk-'".to_string()
            ));
        }
        
        if api_key.len() < 20 {
            return Err(PreviewError::InvalidConfiguration(
                "OpenAI API key appears to be too short".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate Anthropic API key format
    pub fn validate_anthropic_key(api_key: &str) -> Result<(), PreviewError> {
        if api_key.is_empty() {
            return Err(PreviewError::InvalidConfiguration("Anthropic API key cannot be empty".to_string()));
        }
        
        if !api_key.starts_with("sk-ant-") {
            return Err(PreviewError::InvalidConfiguration(
                "Anthropic API key must start with 'sk-ant-'".to_string()
            ));
        }
        
        if api_key.len() < 20 {
            return Err(PreviewError::InvalidConfiguration(
                "Anthropic API key appears to be too short".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate model name for a given provider
    pub fn validate_model_name(provider: &str, model: &str) -> Result<(), PreviewError> {
        match provider.to_lowercase().as_str() {
            "openai" => {
                let valid_models = [
                    "gpt-4", "gpt-4-turbo", "gpt-4o", "gpt-4o-mini",
                    "gpt-3.5-turbo", "gpt-3.5-turbo-16k"
                ];
                
                if !valid_models.iter().any(|&m| model.starts_with(m)) {
                    return Err(PreviewError::InvalidConfiguration(
                        format!("Unknown OpenAI model: {}. Valid models: {}", 
                                model, valid_models.join(", "))
                    ));
                }
            }
            "anthropic" => {
                let valid_models = [
                    "claude-3-opus", "claude-3-sonnet", "claude-3-haiku",
                    "claude-3-5-sonnet", "claude-3-5-haiku"
                ];
                
                if !valid_models.iter().any(|&m| model.starts_with(m)) {
                    return Err(PreviewError::InvalidConfiguration(
                        format!("Unknown Anthropic model: {}. Valid models: {}", 
                                model, valid_models.join(", "))
                    ));
                }
            }
            _ => {
                // For other providers, we don't validate model names
            }
        }
        
        Ok(())
    }
}

/// Configuration helper for LLM providers
pub struct LLMConfig;

impl LLMConfig {
    /// Create OpenAI provider from environment variables
    pub fn openai_from_env() -> Result<crate::OpenAIProvider, PreviewError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| PreviewError::InvalidConfiguration(
                "OPENAI_API_KEY environment variable not set".to_string()
            ))?;
        
        ApiKeyValidator::validate_openai_key(&api_key)?;
        
        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-4o-mini".to_string());
        
        ApiKeyValidator::validate_model_name("openai", &model)?;
        
        Ok(crate::OpenAIProvider::new(api_key).with_model(model))
    }
    
    /// Create Anthropic provider from environment variables
    pub fn anthropic_from_env() -> Result<crate::AnthropicProvider, PreviewError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| PreviewError::InvalidConfiguration(
                "ANTHROPIC_API_KEY environment variable not set".to_string()
            ))?;
        
        ApiKeyValidator::validate_anthropic_key(&api_key)?;
        
        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
        
        ApiKeyValidator::validate_model_name("anthropic", &model)?;
        
        Ok(crate::AnthropicProvider::new(api_key).with_model(model))
    }
    
    /// Create claude-code-api provider from environment variables
    pub fn claude_code_from_env() -> Result<crate::OpenAIProvider, PreviewError> {
        let base_url = std::env::var("CLAUDE_CODE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080/v1".to_string());
        
        let model = std::env::var("CLAUDE_CODE_MODEL")
            .unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string());
        
        // Validate that claude-code-api is accessible
        // Note: This is a synchronous check, in production you might want to make it async
        
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key("not-needed");
        
        Ok(crate::OpenAIProvider::from_config(config, model))
    }
    
    /// Create local provider from environment variables
    pub fn local_from_env() -> Result<crate::LocalProvider, PreviewError> {
        let endpoint = std::env::var("LOCAL_LLM_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        let model = std::env::var("LOCAL_LLM_MODEL")
            .unwrap_or_else(|_| "llama2".to_string());
        
        Ok(crate::LocalProvider::new(endpoint, model))
    }
    
    /// Auto-detect and create the best available provider
    pub async fn auto_detect_provider() -> Result<std::sync::Arc<dyn crate::LLMProvider>, PreviewError> {
        // Try OpenAI first
        if let Ok(provider) = Self::openai_from_env() {
            return Ok(std::sync::Arc::new(provider));
        }
        
        // Try Anthropic
        if let Ok(provider) = Self::anthropic_from_env() {
            return Ok(std::sync::Arc::new(provider));
        }
        
        // Try Claude Code API
        if let Ok(provider) = Self::claude_code_from_env() {
            // Test if the service is accessible
            if Self::test_claude_code_api().await {
                return Ok(std::sync::Arc::new(provider));
            }
        }
        
        // Try Local
        if let Ok(provider) = Self::local_from_env() {
            // Test if the service is accessible
            if Self::test_local_api(&provider).await {
                return Ok(std::sync::Arc::new(provider));
            }
        }
        
        // Fall back to mock provider
        Ok(std::sync::Arc::new(crate::MockProvider::new()))
    }
    
    /// Test if claude-code-api is accessible
    async fn test_claude_code_api() -> bool {
        let base_url = std::env::var("CLAUDE_CODE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        
        if let Ok(response) = reqwest::get(&format!("{}/health", base_url)).await {
            response.status().is_success()
        } else {
            false
        }
    }
    
    /// Test if local LLM API is accessible
    async fn test_local_api(_provider: &crate::LocalProvider) -> bool {
        // This would test the local API endpoint
        // For now, just return false as we can't access provider fields directly
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_key_validation() {
        // Valid key
        assert!(ApiKeyValidator::validate_openai_key("sk-1234567890abcdefghij").is_ok());
        
        // Invalid keys
        assert!(ApiKeyValidator::validate_openai_key("").is_err());
        assert!(ApiKeyValidator::validate_openai_key("invalid").is_err());
        assert!(ApiKeyValidator::validate_openai_key("sk-short").is_err());
    }
    
    #[test]
    fn test_anthropic_key_validation() {
        // Valid key
        assert!(ApiKeyValidator::validate_anthropic_key("sk-ant-1234567890abcdefghij").is_ok());
        
        // Invalid keys
        assert!(ApiKeyValidator::validate_anthropic_key("").is_err());
        assert!(ApiKeyValidator::validate_anthropic_key("sk-1234567890").is_err());
        assert!(ApiKeyValidator::validate_anthropic_key("sk-ant-short").is_err());
    }
    
    #[test]
    fn test_model_validation() {
        // OpenAI models
        assert!(ApiKeyValidator::validate_model_name("openai", "gpt-4").is_ok());
        assert!(ApiKeyValidator::validate_model_name("openai", "gpt-4o-mini").is_ok());
        assert!(ApiKeyValidator::validate_model_name("openai", "invalid-model").is_err());
        
        // Anthropic models
        assert!(ApiKeyValidator::validate_model_name("anthropic", "claude-3-opus-20240229").is_ok());
        assert!(ApiKeyValidator::validate_model_name("anthropic", "claude-3-5-sonnet-20241022").is_ok());
        assert!(ApiKeyValidator::validate_model_name("anthropic", "invalid-model").is_err());
        
        // Other providers (should not validate)
        assert!(ApiKeyValidator::validate_model_name("local", "any-model").is_ok());
    }
}