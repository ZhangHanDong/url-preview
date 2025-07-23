# cc-sdk 测试指南

## 前置条件

1. **安装 Claude CLI**
   ```bash
   npm install -g @anthropic-ai/claude-cli
   ```

2. **认证 Claude**
   ```bash
   claude auth login
   ```

## 快速测试

### 1. 最简单的测试
```bash
cargo run --example cc_sdk_simple_test --features claude-code
```

成功输出示例：
```
🧪 测试 cc-sdk (默认配置)...

测试 1: 最简单调用
✅ 成功!
Hello from cc-sdk!

测试 2: JSON 提取 (仅系统提示)
✅ 成功!
响应: {"status": "ok", "message": "cc-sdk works!"}

测试 3: url-preview 集成
✅ 提取成功: Example Domain
```

### 2. 完整测试
```bash
cargo run --example test_claude_code_direct --features claude-code
```

### 3. 比较测试
```bash
cargo run --example claude_integration_comparison --features claude-code
```

## 重要说明

### 模型名称
cc-sdk 使用模型别名而不是完整的模型名称：
- `"sonnet"` - Claude Sonnet (默认)
- `"haiku"` - Claude Haiku (更快)
- `"opus"` - Claude Opus (更强大)

不要使用像 `"claude-3-sonnet-20240229"` 这样的完整名称。

### 错误排查

1. **"Invalid model name" 错误**
   - 使用模型别名：`"sonnet"`、`"haiku"`、`"opus"`
   - 或者不指定模型，使用默认值

2. **"JSON parsing error" 错误**
   - cc-sdk 可能返回带有额外文本的响应
   - ClaudeCodeProvider 已经实现了智能 JSON 提取

3. **连接错误**
   - 确保 Claude CLI 已安装：`claude --version`
   - 确保已认证：`claude auth status`

## 代码示例

### 基础用法
```rust
use url_preview::{ClaudeCodeProvider, LLMExtractor, Fetcher};
use std::sync::Arc;

let provider = Arc::new(ClaudeCodeProvider::new());
let extractor = LLMExtractor::new(provider);
let fetcher = Fetcher::new();

// 提取数据
let result = extractor.extract::<YourType>(url, &fetcher).await?;
```

### 指定模型
```rust
let provider = Arc::new(
    ClaudeCodeProvider::new()
        .with_haiku()  // 使用 Haiku 模型
);
```

### 自定义系统提示
```rust
let provider = Arc::new(
    ClaudeCodeProvider::new()
        .with_system_prompt("Your custom prompt".to_string())
);
```

## 性能建议

1. **开发阶段**：使用 `haiku` 模型，响应更快
2. **生产环境**：根据需求选择合适的模型
3. **本地测试**：cc-sdk 直接调用 CLI，没有网络延迟

## 与 claude-code-api 的区别

| 特性 | cc-sdk | claude-code-api |
|------|--------|-----------------|
| 实现方式 | 直接调用 CLI | HTTP API |
| 性能 | 更快 | 稍慢 |
| 配置 | 简单 | 需要启动服务 |
| 适用场景 | 本地开发 | 服务化部署 |