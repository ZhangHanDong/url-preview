# 测试指南 - Browser 和 LLM 功能

本指南帮助您测试 url-preview 0.6.0 版本新增的浏览器渲染和 LLM 数据提取功能。

## 前置准备

### 1. 安装 Node.js
浏览器功能需要 Node.js 来运行 playwright-mcp：
```bash
# macOS
brew install node

# 或者从 https://nodejs.org 下载安装
```

### 2. 验证环境
```bash
node --version  # 应该显示 v16 或更高版本
npm --version   # 应该显示 8 或更高版本
```

## 一、测试基础功能

### 1. 运行单元测试
```bash
# 测试所有功能
cargo test --all-features

# 只测试浏览器功能
cargo test --features browser

# 只测试 LLM 功能
cargo test --features llm

# 测试特定模块
cargo test browser_tests --features browser
cargo test llm_tests --features llm
```

### 2. 运行集成测试
```bash
# 运行所有集成测试
cargo test --test '*' --all-features
```

## 二、测试浏览器功能

### 1. 基础浏览器预览示例
```bash
# 运行浏览器预览示例
cargo run --example browser_preview --features browser
```

这个示例会：
- 初始化 playwright-mcp 服务器
- 测试多个网站（静态和动态）
- 显示是否使用了浏览器渲染
- 演示不同的浏览器使用策略（Always/Never/Auto）

### 2. 手动测试浏览器功能
创建测试文件 `test_browser.rs`：

```rust
use url_preview::{
    BrowserUsagePolicy, McpConfig, PreviewService, PreviewServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置浏览器
    let mcp_config = McpConfig {
        enabled: true,
        server_command: vec![
            "npx".to_string(),
            "-y".to_string(),
            "@modelcontextprotocol/server-playwright".to_string(),
        ],
        browser_timeout: 30,
        ..Default::default()
    };
    
    let config = PreviewServiceConfig::new(1000)
        .with_mcp_config(mcp_config)
        .with_browser_usage_policy(BrowserUsagePolicy::Auto);
    
    let service = PreviewService::new_with_config(config);
    
    // 测试需要浏览器的网站
    let test_urls = vec![
        "https://twitter.com/rustlang",     // SPA
        "https://reddit.com/r/rust",        // SPA
        "https://www.rust-lang.org",        // 静态网站
    ];
    
    for url in test_urls {
        println!("\n测试 URL: {}", url);
        match service.generate_preview(url).await {
            Ok(preview) => {
                println!("✅ 成功获取预览");
                println!("  标题: {:?}", preview.title);
                println!("  描述: {:?}", preview.description.map(|d| {
                    if d.len() > 100 { format!("{}...", &d[..100]) } else { d }
                }));
            }
            Err(e) => println!("❌ 错误: {}", e),
        }
    }
    
    Ok(())
}
```

### 3. 测试截图功能
```rust
use url_preview::{BrowserPreviewService, BrowserUsagePolicy, McpConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser_service = BrowserPreviewService::new(
        McpConfig { enabled: true, ..Default::default() },
        BrowserUsagePolicy::Always,
    );
    
    browser_service.initialize().await?;
    
    // 截图
    match browser_service.browser_fetcher.take_screenshot("https://rust-lang.org").await {
        Ok(data) => {
            // 保存截图
            std::fs::write("screenshot.png", data)?;
            println!("截图已保存到 screenshot.png");
        }
        Err(e) => println!("截图失败: {}", e),
    }
    
    Ok(())
}
```

## 三、测试 LLM 功能

### 1. 运行 LLM 提取示例
```bash
# 使用 Mock Provider（不需要 API key）
cargo run --example llm_extraction --features llm
```

### 2. 测试结构化数据提取
创建测试文件 `test_llm.rs`：

```rust
use url_preview::{Fetcher, LLMExtractor, MockProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct WebsiteInfo {
    title: String,
    main_topic: String,
    key_features: Vec<String>,
    programming_language: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 Mock Provider 进行测试
    let provider = Arc::new(MockProvider::new());
    let extractor = LLMExtractor::new(provider);
    let fetcher = Fetcher::new();
    
    // 测试提取
    match extractor.extract::<WebsiteInfo>("https://www.rust-lang.org", &fetcher).await {
        Ok(result) => {
            println!("提取成功!");
            println!("模型: {}", result.model);
            println!("数据: {:?}", result.data);
        }
        Err(e) => println!("提取失败: {}", e),
    }
    
    Ok(())
}
```

### 3. 测试不同内容格式
```rust
use url_preview::{ContentFormat, LLMExtractorConfig};

// 测试 Markdown 格式
let config = LLMExtractorConfig {
    format: ContentFormat::Markdown,
    clean_html: true,
    max_content_length: 10_000,
    ..Default::default()
};

let extractor = LLMExtractor::with_config(provider, config);
```

## 四、测试组合功能

### 1. 浏览器 + LLM 组合
```bash
cargo run --example browser_llm_extraction --features "browser llm"
```

这个示例展示了：
- 使用浏览器获取 JavaScript 渲染的内容
- 使用 LLM 从渲染后的内容提取结构化数据
- 自定义 JavaScript 执行和数据提取

## 五、性能测试

### 1. 运行基准测试
```bash
# 运行所有基准测试
cargo bench

# 测试浏览器性能
cargo bench --features browser browser_benchmark
```

### 2. 并发测试
```rust
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let service = PreviewService::new();
    let urls = vec![
        "https://example1.com",
        "https://example2.com",
        "https://example3.com",
    ];
    
    let start = std::time::Instant::now();
    let futures = urls.iter().map(|url| service.generate_preview(url));
    let results = join_all(futures).await;
    
    println!("处理 {} 个 URL 耗时: {:?}", urls.len(), start.elapsed());
}
```

## 六、故障排除

### 1. 浏览器功能问题

**问题**: MCP 服务器启动失败
```
Error: Failed to start MCP server
```
**解决方案**:
- 确保 Node.js 已安装
- 检查网络连接（npx 需要下载包）
- 手动运行: `npx @modelcontextprotocol/server-playwright`

**问题**: 浏览器超时
```
Error: Browser operation timeout
```
**解决方案**:
- 增加超时时间: `browser_timeout: 60`
- 检查目标网站是否可访问
- 查看是否有防爬虫机制

### 2. LLM 功能问题

**问题**: Schema 验证失败
```
Error: JSON parsing error
```
**解决方案**:
- 确保结构体实现了必要的 trait: `Serialize, Deserialize, JsonSchema`
- 检查字段类型是否匹配
- 使用 Option<T> 处理可选字段

### 3. 调试技巧

启用日志：
```bash
# 设置日志级别
RUST_LOG=url_preview=debug cargo run --example browser_preview --features "browser logging"

# 只看特定模块
RUST_LOG=url_preview::browser_fetcher=debug cargo run
```

## 七、真实 API 测试（可选）

### 使用 OpenAI
```rust
#[cfg(feature = "async-openai")]
use url_preview::llm_providers::openai::OpenAIProvider;

let provider = Arc::new(OpenAIProvider::new("your-api-key".to_string()));
let extractor = LLMExtractor::new(provider);
```

### 测试命令
```bash
# 设置 API key
export OPENAI_API_KEY="your-key"

# 运行测试
cargo run --example llm_extraction --features "llm async-openai"
```

## 八、检查清单

- [ ] 单元测试全部通过
- [ ] 浏览器示例正常运行
- [ ] LLM 示例正常运行
- [ ] 组合示例正常运行
- [ ] SPA 网站能正确获取内容
- [ ] 静态网站不会使用浏览器（Auto 模式）
- [ ] Mock Provider 返回预期数据
- [ ] 错误处理正常工作
- [ ] 性能符合预期

## 总结

完成以上测试后，您应该对新功能有全面的了解。如果遇到问题：

1. 检查日志输出
2. 确认依赖正确安装
3. 查看示例代码
4. 参考错误信息进行调试

记住，浏览器功能需要 Node.js，而 LLM 功能在使用真实 API 时需要相应的 API key。