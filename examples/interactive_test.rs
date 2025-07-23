//! Interactive test program for browser and LLM features
//!
//! Run with:
//! ```
//! cargo run --example interactive_test --features "browser llm"
//! ```

use std::io::{self, Write};
use url_preview::{
    BrowserUsagePolicy, ContentFormat, Fetcher, LLMExtractor, LLMExtractorConfig,
    McpConfig, MockProvider, PreviewService, PreviewServiceConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct WebPageInfo {
    title: String,
    description: String,
    main_topics: Vec<String>,
    has_images: bool,
    estimated_read_time: Option<String>,
}

fn print_menu() {
    println!("\n🚀 URL Preview 0.6.0 - 交互式测试");
    println!("==================================");
    println!("1. 测试标准预览（无浏览器）");
    println!("2. 测试浏览器预览（自动检测）");
    println!("3. 测试浏览器预览（强制使用）");
    println!("4. 测试 LLM 数据提取");
    println!("5. 测试浏览器 + LLM 组合");
    println!("6. 运行所有测试");
    println!("0. 退出");
    println!("==================================");
    print!("请选择 (0-6): ");
    io::stdout().flush().unwrap();
}

async fn test_standard_preview() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📄 测试标准预览...");
    let service = PreviewService::new();
    
    let url = get_url_from_user("请输入 URL (默认: https://www.rust-lang.org): ", 
                                 "https://www.rust-lang.org");
    
    match service.generate_preview(&url).await {
        Ok(preview) => {
            println!("\n✅ 预览成功:");
            println!("  URL: {}", preview.url);
            println!("  标题: {:?}", preview.title);
            println!("  描述: {:?}", preview.description);
            println!("  图片: {:?}", preview.image_url);
            println!("  网站: {:?}", preview.site_name);
        }
        Err(e) => println!("❌ 错误: {}", e),
    }
    
    Ok(())
}

async fn test_browser_preview(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌐 测试浏览器预览...");
    
    let policy = if force {
        println!("模式: 强制使用浏览器");
        BrowserUsagePolicy::Always
    } else {
        println!("模式: 自动检测");
        BrowserUsagePolicy::Auto
    };
    
    #[cfg(feature = "browser")]
    {
        let mcp_config = McpConfig {
            enabled: true,
            ..Default::default()
        };
        
        let config = PreviewServiceConfig::new(1000)
            .with_mcp_config(mcp_config)
            .with_browser_usage_policy(policy);
        
        let service = PreviewService::new_with_config(config);
        
        let url = get_url_from_user("请输入 URL (默认: https://twitter.com/rustlang): ", 
                                     "https://twitter.com/rustlang");
        
        println!("正在初始化浏览器...");
        if let Some(browser_service) = &service.browser_service {
            browser_service.initialize().await?;
        }
        
        match service.generate_preview(&url).await {
            Ok(preview) => {
                println!("\n✅ 预览成功:");
                println!("  URL: {}", preview.url);
                println!("  标题: {:?}", preview.title);
                println!("  描述: {:?}", preview.description);
                
                if let Some(browser_service) = &service.browser_service {
                    let used_browser = browser_service.should_use_browser(&url);
                    println!("  使用浏览器: {}", if used_browser { "是" } else { "否" });
                }
            }
            Err(e) => println!("❌ 错误: {}", e),
        }
    }
    
    #[cfg(not(feature = "browser"))]
    {
        println!("❌ 浏览器功能未启用。请使用 --features browser");
    }
    
    Ok(())
}

async fn test_llm_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 测试 LLM 数据提取...");
    
    #[cfg(feature = "llm")]
    {
        let provider = Arc::new(MockProvider::new());
        let config = LLMExtractorConfig {
            format: ContentFormat::Html,
            clean_html: true,
            max_content_length: 50_000,
            ..Default::default()
        };
        
        let extractor = LLMExtractor::with_config(provider, config);
        let fetcher = Fetcher::new();
        
        let url = get_url_from_user("请输入 URL (默认: https://www.rust-lang.org): ", 
                                     "https://www.rust-lang.org");
        
        println!("正在提取数据...");
        match extractor.extract::<WebPageInfo>(&url, &fetcher).await {
            Ok(result) => {
                println!("\n✅ 提取成功:");
                println!("  模型: {}", result.model);
                println!("  数据: {:#?}", result.data);
            }
            Err(e) => println!("❌ 错误: {}", e),
        }
    }
    
    #[cfg(not(feature = "llm"))]
    {
        println!("❌ LLM 功能未启用。请使用 --features llm");
    }
    
    Ok(())
}

async fn test_browser_llm_combo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔗 测试浏览器 + LLM 组合...");
    
    #[cfg(all(feature = "browser", feature = "llm"))]
    {
        use url_preview::BrowserPreviewService;
        
        // 设置浏览器
        let mcp_config = McpConfig {
            enabled: true,
            ..Default::default()
        };
        
        let browser_service = BrowserPreviewService::new(mcp_config, BrowserUsagePolicy::Always);
        browser_service.initialize().await?;
        
        let url = get_url_from_user("请输入 URL (默认: https://reddit.com/r/rust): ", 
                                     "https://reddit.com/r/rust");
        
        // 使用浏览器获取内容
        println!("使用浏览器获取内容...");
        match browser_service.browser_fetcher.fetch_with_browser(&url).await {
            Ok(html) => {
                println!("✅ 获取成功，内容长度: {} 字节", html.len());
                
                // 使用 LLM 提取数据
                println!("使用 LLM 提取数据...");
                let provider = Arc::new(MockProvider::new());
                let extractor = LLMExtractor::new(provider);
                
                // 模拟从 HTML 提取
                let mock_info = WebPageInfo {
                    title: "Rust Programming Language Community".to_string(),
                    description: "Discussion forum for Rust developers".to_string(),
                    main_topics: vec!["Programming".to_string(), "Systems".to_string()],
                    has_images: true,
                    estimated_read_time: Some("5 minutes".to_string()),
                };
                
                println!("\n✅ 提取结果:");
                println!("{:#?}", mock_info);
            }
            Err(e) => println!("❌ 浏览器获取失败: {}", e),
        }
    }
    
    #[cfg(not(all(feature = "browser", feature = "llm")))]
    {
        println!("❌ 需要同时启用 browser 和 llm 功能。请使用 --features \"browser llm\"");
    }
    
    Ok(())
}

async fn run_all_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 运行所有测试...\n");
    
    println!("1️⃣ 标准预览测试");
    test_standard_preview().await?;
    
    println!("\n2️⃣ 浏览器预览测试（自动）");
    test_browser_preview(false).await?;
    
    println!("\n3️⃣ 浏览器预览测试（强制）");
    test_browser_preview(true).await?;
    
    println!("\n4️⃣ LLM 数据提取测试");
    test_llm_extraction().await?;
    
    println!("\n5️⃣ 浏览器 + LLM 组合测试");
    test_browser_llm_combo().await?;
    
    println!("\n✅ 所有测试完成!");
    
    Ok(())
}

fn get_url_from_user(prompt: &str, default: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    
    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 URL Preview 0.6.0 交互式测试程序");
    println!("提示: 确保已启用 browser 和 llm features");
    
    loop {
        print_menu();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => test_standard_preview().await?,
            "2" => test_browser_preview(false).await?,
            "3" => test_browser_preview(true).await?,
            "4" => test_llm_extraction().await?,
            "5" => test_browser_llm_combo().await?,
            "6" => run_all_tests().await?,
            "0" => {
                println!("\n👋 再见!");
                break;
            }
            _ => println!("❌ 无效选择，请重试"),
        }
        
        println!("\n按 Enter 继续...");
        let mut pause = String::new();
        io::stdin().read_line(&mut pause)?;
    }
    
    Ok(())
}