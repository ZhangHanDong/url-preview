//! LLM-based structured data extraction module
//!
//! This module provides functionality to extract structured data from web pages
//! using Large Language Models (LLMs). It supports multiple LLM providers and
//! various content preprocessing formats.

use crate::{PreviewError, Fetcher};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "cache")]
use crate::Cache;

/// Configuration for LLM extraction
#[derive(Clone, Debug)]
pub struct LLMExtractorConfig {
    /// Content format for preprocessing
    pub format: ContentFormat,
    /// Whether to clean HTML before processing
    pub clean_html: bool,
    /// Maximum content length to send to LLM
    pub max_content_length: usize,
    /// Model-specific parameters
    pub model_params: HashMap<String, Value>,
}

impl Default for LLMExtractorConfig {
    fn default() -> Self {
        Self {
            format: ContentFormat::Html,
            clean_html: true,
            max_content_length: 50_000, // 50KB default
            model_params: HashMap::new(),
        }
    }
}

/// Content format for preprocessing
#[derive(Clone, Debug, PartialEq)]
pub enum ContentFormat {
    /// Raw HTML content
    Html,
    /// HTML converted to Markdown
    Markdown,
    /// Clean text extracted from HTML
    Text,
    /// Screenshot of the page (for multi-modal models)
    Image,
}

/// Processed content ready for LLM
#[derive(Clone, Debug)]
pub struct ProcessedContent {
    /// The processed content
    pub content: String,
    /// Format of the content
    pub format: ContentFormat,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result of LLM extraction
#[derive(Clone, Debug)]
pub struct ExtractionResult<T> {
    /// Extracted data
    pub data: T,
    /// LLM model used
    pub model: String,
    /// Token usage information
    pub usage: Option<TokenUsage>,
}

/// Token usage information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Get the name of the provider
    fn name(&self) -> &str;
    
    /// Generate structured data from content
    async fn generate(
        &self,
        prompt: String,
        schema: Value,
        config: &LLMExtractorConfig,
    ) -> Result<Value, PreviewError>;
    
    /// Stream structured data from content (optional)
    async fn stream(
        &self,
        _prompt: String,
        _schema: Value,
        _config: &LLMExtractorConfig,
    ) -> Result<Box<dyn futures::Stream<Item = Result<Value, PreviewError>> + Send + Unpin>, PreviewError> {
        Err(PreviewError::UnsupportedOperation("Streaming not supported by this provider".into()))
    }
}

/// Main LLM extractor
pub struct LLMExtractor {
    /// LLM provider
    provider: Arc<dyn LLMProvider>,
    /// Content preprocessor
    preprocessor: ContentPreprocessor,
    /// Configuration
    config: LLMExtractorConfig,
    /// Optional cache
    #[cfg(feature = "cache")]
    cache: Option<Arc<Cache>>,
}

impl LLMExtractor {
    /// Create a new LLM extractor
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self {
            provider,
            preprocessor: ContentPreprocessor::new(),
            config: LLMExtractorConfig::default(),
            #[cfg(feature = "cache")]
            cache: None,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(provider: Arc<dyn LLMProvider>, config: LLMExtractorConfig) -> Self {
        Self {
            provider,
            preprocessor: ContentPreprocessor::new(),
            config,
            #[cfg(feature = "cache")]
            cache: None,
        }
    }
    
    /// Set cache
    #[cfg(feature = "cache")]
    pub fn with_cache(mut self, cache: Arc<Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
    
    /// Extract structured data from a URL
    pub async fn extract<T>(&self, url: &str, fetcher: &Fetcher) -> Result<ExtractionResult<T>, PreviewError>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + schemars::JsonSchema,
    {
        // Check cache first
        #[cfg(feature = "cache")]
        if let Some(cache) = &self.cache {
            let cache_key = format!("llm:{}:{}", url, std::any::type_name::<T>());
            if let Some(cached) = cache.get(&cache_key).await {
                if let Ok(result) = serde_json::from_str::<T>(&cached.description.unwrap_or_default()) {
                    return Ok(ExtractionResult {
                        data: result,
                        model: "cached".to_string(),
                        usage: None,
                    });
                }
            }
        }
        
        // Fetch content
        let fetch_result = fetcher.fetch(url).await?;
        let html = match fetch_result {
            crate::FetchResult::Html(h) => h,
            _ => return Err(PreviewError::InvalidContentType("Expected HTML".to_string())),
        };
        
        // Preprocess content
        let processed = self.preprocessor.preprocess(&html, &self.config).await?;
        
        // Generate schema
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema)?;
        
        // Build prompt
        let prompt = self.build_prompt(&processed, &schema_json)?;
        
        // Call LLM
        let result = self.provider.generate(prompt, schema_json, &self.config).await?;
        
        // Parse result
        let extracted: T = serde_json::from_value(result)?;
        
        // Cache result
        #[cfg(feature = "cache")]
        if let Some(cache) = &self.cache {
            let cache_key = format!("llm:{}:{}", url, std::any::type_name::<T>());
            let preview = crate::Preview {
                url: url.to_string(),
                title: Some(format!("LLM Extraction: {}", std::any::type_name::<T>())),
                description: Some(serde_json::to_string(&extracted)?),
                image_url: None,
                site_name: None,
                favicon: None,
            };
            cache.set(cache_key, preview).await;
        }
        
        Ok(ExtractionResult {
            data: extracted,
            model: self.provider.name().to_string(),
            usage: None, // TODO: Get from provider
        })
    }
    
    /// Build prompt for LLM
    fn build_prompt(&self, content: &ProcessedContent, schema: &Value) -> Result<String, PreviewError> {
        let schema_str = serde_json::to_string_pretty(schema)?;
        
        let format_hint = match content.format {
            ContentFormat::Html => "HTML",
            ContentFormat::Markdown => "Markdown",
            ContentFormat::Text => "plain text",
            ContentFormat::Image => "image",
        };
        
        Ok(format!(
            "Extract structured data from the following {} content according to this schema:\n\n\
            Schema:\n```json\n{}\n```\n\n\
            Content:\n{}\n\n\
            Extract the data and return it as a valid JSON object matching the schema.",
            format_hint,
            schema_str,
            content.content
        ))
    }
}

/// Content preprocessor
pub struct ContentPreprocessor {
    html_cleaner: HtmlCleaner,
}

impl ContentPreprocessor {
    pub fn new() -> Self {
        Self {
            html_cleaner: HtmlCleaner::new(),
        }
    }
    
    /// Preprocess HTML content
    pub async fn preprocess(&self, html: &str, config: &LLMExtractorConfig) -> Result<ProcessedContent, PreviewError> {
        let processed_html = if config.clean_html {
            self.html_cleaner.clean(html)?
        } else {
            html.to_string()
        };
        
        let content = match config.format {
            ContentFormat::Html => processed_html,
            ContentFormat::Markdown => self.convert_to_markdown(&processed_html)?,
            ContentFormat::Text => self.extract_text(&processed_html)?,
            ContentFormat::Image => {
                return Err(PreviewError::UnsupportedOperation("Image format not yet implemented".into()));
            }
        };
        
        // Truncate if needed
        let content = if content.len() > config.max_content_length {
            content.chars().take(config.max_content_length).collect()
        } else {
            content
        };
        
        Ok(ProcessedContent {
            content,
            format: config.format.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Convert HTML to Markdown
    fn convert_to_markdown(&self, html: &str) -> Result<String, PreviewError> {
        // Simple conversion - in production, use a proper HTML to Markdown converter
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
        let mut markdown = String::new();
        
        // Extract title
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title) = document.select(&title_selector).next() {
                markdown.push_str(&format!("# {}\n\n", title.text().collect::<String>()));
            }
        }
        
        // Extract headings
        for i in 1..=6 {
            if let Ok(selector) = Selector::parse(&format!("h{}", i)) {
                for element in document.select(&selector) {
                    let heading_level = "#".repeat(i);
                    markdown.push_str(&format!("{} {}\n\n", heading_level, element.text().collect::<String>()));
                }
            }
        }
        
        // Extract paragraphs
        if let Ok(p_selector) = Selector::parse("p") {
            for element in document.select(&p_selector) {
                markdown.push_str(&format!("{}\n\n", element.text().collect::<String>()));
            }
        }
        
        Ok(markdown)
    }
    
    /// Extract clean text from HTML
    fn extract_text(&self, html: &str) -> Result<String, PreviewError> {
        use scraper::Html;
        
        let document = Html::parse_document(html);
        let text = document.root_element().text().collect::<Vec<_>>().join(" ");
        
        // Clean up whitespace
        let text = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(text)
    }
}

/// HTML cleaner
struct HtmlCleaner;

impl HtmlCleaner {
    fn new() -> Self {
        Self
    }
    
    /// Clean HTML by removing unnecessary elements
    fn clean(&self, html: &str) -> Result<String, PreviewError> {
        use scraper::{Html, Selector};
        use std::collections::HashSet;
        
        let document = Html::parse_document(html);
        
        // Elements to remove completely
        let remove_selectors = vec![
            "script", "style", "noscript", "iframe", "object", "embed", 
            "form", "input", "button", "select", "textarea", "option",
            "nav", "header", "footer", "aside", "menu", "menuitem",
            "audio", "video", "source", "track", "canvas", "svg",
            "meta", "link", "base", "title"
        ];
        
        // Elements to keep but clean attributes (for future use)
        let _content_selectors = vec![
            "h1", "h2", "h3", "h4", "h5", "h6",
            "p", "div", "span", "section", "article", "main",
            "ul", "ol", "li", "dl", "dt", "dd",
            "table", "thead", "tbody", "tr", "th", "td",
            "blockquote", "pre", "code",
            "strong", "b", "em", "i", "u", "mark",
            "a", "img", "br", "hr"
        ];
        
        let mut cleaned_html = String::new();
        let mut removed_tags: HashSet<String> = HashSet::new();
        
        // Process the HTML to extract clean content
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                cleaned_html = self.extract_clean_content(body, &remove_selectors, &mut removed_tags);
            } else {
                // If no body tag, process the entire document
                cleaned_html = self.extract_clean_content(document.root_element(), &remove_selectors, &mut removed_tags);
            }
        }
        
        // If nothing was extracted, fall back to simple text extraction
        if cleaned_html.trim().is_empty() {
            cleaned_html = document.root_element().text().collect::<Vec<_>>().join(" ");
            // Clean up excessive whitespace
            cleaned_html = cleaned_html
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
        }
        
        Ok(cleaned_html)
    }
    
    /// Recursively extract clean content from an element
    fn extract_clean_content(
        &self, 
        element: scraper::ElementRef, 
        remove_selectors: &[&str],
        removed_tags: &mut std::collections::HashSet<String>
    ) -> String {
        use scraper::{Node, ElementRef};
        
        let mut content = String::new();
        let tag_name = element.value().name();
        
        // Skip if this is a tag we want to remove
        if remove_selectors.contains(&tag_name) {
            removed_tags.insert(tag_name.to_string());
            return content;
        }
        
        // Process child nodes
        for child in element.children() {
            match child.value() {
                Node::Text(text) => {
                    let text_content = text.text.trim();
                    if !text_content.is_empty() {
                        content.push_str(text_content);
                        content.push(' ');
                    }
                }
                Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        let child_content = self.extract_clean_content(child_element, remove_selectors, removed_tags);
                        if !child_content.trim().is_empty() {
                            // Add some structure for block elements
                            match child_element.value().name() {
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                                    content.push_str(&format!("\n\n{}\n", child_content.trim()));
                                }
                                "p" | "div" | "section" | "article" => {
                                    content.push_str(&format!("\n{}\n", child_content.trim()));
                                }
                                "li" => {
                                    content.push_str(&format!("• {}\n", child_content.trim()));
                                }
                                "br" => {
                                    content.push('\n');
                                }
                                "hr" => {
                                    content.push_str("\n---\n");
                                }
                                _ => {
                                    content.push_str(&child_content);
                                }
                            }
                        }
                    }
                }
                _ => {} // Ignore comments and other node types
            }
        }
        
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_format() {
        assert_eq!(ContentFormat::Html, ContentFormat::Html);
        assert_ne!(ContentFormat::Html, ContentFormat::Markdown);
    }
    
    #[test]
    fn test_default_config() {
        let config = LLMExtractorConfig::default();
        assert_eq!(config.format, ContentFormat::Html);
        assert!(config.clean_html);
        assert_eq!(config.max_content_length, 50_000);
    }
}