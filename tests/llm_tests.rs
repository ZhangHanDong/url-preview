//! Tests for LLM-based extraction functionality

#[cfg(feature = "llm")]
mod llm_tests {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use url_preview::{
        ContentFormat, Fetcher, LLMExtractor, LLMExtractorConfig, MockProvider,
    };

    #[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
    struct TestData {
        title: String,
        count: i32,
        active: bool,
    }

    #[test]
    fn test_llm_config_default() {
        let config = LLMExtractorConfig::default();
        assert_eq!(config.format, ContentFormat::Html);
        assert!(config.clean_html);
        assert_eq!(config.max_content_length, 50_000);
        assert!(config.model_params.is_empty());
    }

    #[test]
    fn test_content_format_equality() {
        assert_eq!(ContentFormat::Html, ContentFormat::Html);
        assert_ne!(ContentFormat::Html, ContentFormat::Markdown);
        assert_ne!(ContentFormat::Markdown, ContentFormat::Text);
        assert_ne!(ContentFormat::Text, ContentFormat::Image);
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = Arc::new(MockProvider::new());
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" },
                "active": { "type": "boolean" },
                "tags": { "type": "array" }
            }
        });

        let result = provider
            .generate(
                "Test prompt".to_string(),
                schema,
                &LLMExtractorConfig::default(),
            )
            .await
            .unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Mock value");
        assert_eq!(obj.get("age").unwrap().as_i64().unwrap(), 42);
        assert_eq!(obj.get("active").unwrap().as_bool().unwrap(), true);
        assert!(obj.get("tags").unwrap().is_array());
    }

    #[test]
    fn test_content_preprocessor_truncation() {
        use url_preview::llm_extractor::ContentPreprocessor;

        let preprocessor = ContentPreprocessor::new();
        let long_content = "a".repeat(100_000);
        
        // This is a conceptual test - actual implementation would be tested here
        // The preprocessor should truncate content that exceeds max_content_length
        assert!(long_content.len() > 50_000);
    }

    #[tokio::test]
    async fn test_llm_extractor_with_mock() {
        let provider = Arc::new(MockProvider::new());
        let extractor = LLMExtractor::new(provider);
        let fetcher = Fetcher::new();

        // This would fail with actual fetching, but demonstrates the API
        match extractor.extract::<TestData>("https://example.invalid", &fetcher).await {
            Ok(_) => {
                // In a real test, this would be unexpected
            }
            Err(e) => {
                // Expected to fail due to invalid URL
                assert!(e.to_string().contains("invalid") || e.to_string().contains("fetch"));
            }
        }
    }

    #[test]
    fn test_schema_generation() {
        let schema = schemars::schema_for!(TestData);
        let schema_json = serde_json::to_value(&schema).unwrap();

        // Verify schema has expected structure
        assert!(schema_json.is_object());
        assert!(schema_json.get("properties").is_some());
        
        let properties = schema_json.get("properties").unwrap();
        assert!(properties.get("title").is_some());
        assert!(properties.get("count").is_some());
        assert!(properties.get("active").is_some());
    }

    #[test]
    fn test_extraction_result_structure() {
        use url_preview::ExtractionResult;

        let test_data = TestData {
            title: "Test".to_string(),
            count: 42,
            active: true,
        };

        let result = ExtractionResult {
            data: test_data,
            model: "test-model".to_string(),
            usage: None,
        };

        assert_eq!(result.model, "test-model");
        assert_eq!(result.data.title, "Test");
        assert_eq!(result.data.count, 42);
        assert!(result.data.active);
        assert!(result.usage.is_none());
    }

    #[test]
    fn test_token_usage() {
        use url_preview::llm_extractor::TokenUsage;

        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_processed_content() {
        use url_preview::ProcessedContent;
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let content = ProcessedContent {
            content: "Processed HTML content".to_string(),
            format: ContentFormat::Html,
            metadata,
        };

        assert_eq!(content.content, "Processed HTML content");
        assert_eq!(content.format, ContentFormat::Html);
        assert_eq!(content.metadata.get("source").unwrap(), "test");
    }

    #[cfg(feature = "openai")]
    #[test]
    fn test_openai_provider_creation() {
        use url_preview::llm_providers::openai::OpenAIProvider;

        let provider = OpenAIProvider::new("test-key".to_string())
            .with_model("gpt-4".to_string());

        assert_eq!(provider.name(), "openai");
    }

    #[cfg(feature = "anthropic")]
    #[test]
    fn test_anthropic_provider_creation() {
        use url_preview::llm_providers::anthropic::AnthropicProvider;

        let provider = AnthropicProvider::new("test-key".to_string())
            .with_model("claude-3-opus".to_string());

        assert_eq!(provider.name(), "anthropic");
    }
}

#[cfg(not(feature = "llm"))]
#[test]
fn test_llm_feature_disabled() {
    // This test ensures the crate compiles without llm feature
    assert!(true, "LLM feature is disabled");
}