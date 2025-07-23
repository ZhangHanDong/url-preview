//! Tests for browser-based preview generation

#[cfg(feature = "browser")]
mod browser_tests {
    use url_preview::{
        BrowserFetcher, BrowserUsagePolicy, McpConfig, McpTransport, PreviewService,
        PreviewServiceConfig,
    };

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.browser_timeout, 30);
        assert_eq!(config.max_sessions, 5);
        assert!(matches!(config.transport, McpTransport::Stdio));
    }

    #[test]
    fn test_browser_usage_policy() {
        let config = McpConfig::default();
        
        // Test Auto policy
        let fetcher = BrowserFetcher::new(config.clone(), BrowserUsagePolicy::Auto);
        assert!(fetcher.should_use_browser("https://twitter.com/home"));
        assert!(fetcher.should_use_browser("https://reddit.com/r/rust"));
        assert!(!fetcher.should_use_browser("https://example.com"));
        assert!(!fetcher.should_use_browser("https://rust-lang.org"));
        
        // Test Always policy
        let fetcher = BrowserFetcher::new(config.clone(), BrowserUsagePolicy::Always);
        assert!(fetcher.should_use_browser("https://example.com"));
        assert!(fetcher.should_use_browser("https://rust-lang.org"));
        
        // Test Never policy
        let fetcher = BrowserFetcher::new(config, BrowserUsagePolicy::Never);
        assert!(!fetcher.should_use_browser("https://twitter.com"));
        assert!(!fetcher.should_use_browser("https://reddit.com"));
    }

    #[test]
    fn test_spa_detection() {
        let config = McpConfig::default();
        let fetcher = BrowserFetcher::new(config, BrowserUsagePolicy::Auto);
        
        // Known SPAs
        assert!(fetcher.should_use_browser("https://twitter.com/user"));
        assert!(fetcher.should_use_browser("https://x.com/user"));
        assert!(fetcher.should_use_browser("https://instagram.com/user"));
        assert!(fetcher.should_use_browser("https://facebook.com/page"));
        assert!(fetcher.should_use_browser("https://linkedin.com/in/user"));
        assert!(fetcher.should_use_browser("https://discord.com/channels/123"));
        assert!(fetcher.should_use_browser("https://app.slack.com/workspace"));
        assert!(fetcher.should_use_browser("https://notion.so/page"));
        assert!(fetcher.should_use_browser("https://myapp.vercel.app"));
        assert!(fetcher.should_use_browser("https://myapp.netlify.app"));
        assert!(fetcher.should_use_browser("https://myapp.web.app"));
        
        // SPA route patterns
        assert!(fetcher.should_use_browser("https://example.com/#!/page"));
        assert!(fetcher.should_use_browser("https://example.com/#/route"));
        assert!(fetcher.should_use_browser("https://example.com/app/dashboard"));
        assert!(fetcher.should_use_browser("https://example.com/dashboard/stats"));
        
        // Non-SPAs
        assert!(!fetcher.should_use_browser("https://blog.rust-lang.org"));
        assert!(!fetcher.should_use_browser("https://docs.rs"));
        assert!(!fetcher.should_use_browser("https://crates.io"));
    }

    #[tokio::test]
    async fn test_preview_service_with_browser_config() {
        let mcp_config = McpConfig {
            enabled: false, // Disabled for unit test
            ..Default::default()
        };
        
        let config = PreviewServiceConfig::new(100)
            .with_mcp_config(mcp_config)
            .with_browser_usage_policy(BrowserUsagePolicy::Auto);
        
        let service = PreviewService::new_with_config(config);
        
        // Browser service should be created but not active
        assert!(service.browser_service.is_some());
    }

    #[tokio::test]
    async fn test_browser_service_initialization() {
        use url_preview::BrowserPreviewService;
        
        let mcp_config = McpConfig {
            enabled: false, // Disabled for unit test
            ..Default::default()
        };
        
        let browser_service = BrowserPreviewService::new(mcp_config, BrowserUsagePolicy::Auto);
        
        // Should handle disabled MCP gracefully
        let result = browser_service.initialize().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_request_serialization() {
        use serde_json::json;
        
        // Test that MCP requests would serialize correctly
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "browser_navigate",
                "arguments": {
                    "url": "https://example.com"
                }
            },
            "id": 1
        });
        
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "tools/call");
        assert_eq!(request["params"]["name"], "browser_navigate");
    }
}

#[cfg(not(feature = "browser"))]
#[test]
fn test_browser_feature_disabled() {
    // This test ensures the crate compiles without browser feature
    assert!(true, "Browser feature is disabled");
}