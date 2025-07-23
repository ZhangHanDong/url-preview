# LLM 结构化数据提取测试指南

## 当前可用的测试方案

由于 claude-code-api 存在响应格式兼容性问题，建议使用以下方案进行测试：

### 1. 使用 Anthropic API (推荐)

如果你有 Anthropic API Key，这是最直接的方案：

```bash
# 设置 API Key
export ANTHROPIC_API_KEY="your-anthropic-api-key"

# 运行测试
cargo run --example test_anthropic_direct --features llm
```

**优点**：
- 原生支持，无兼容性问题
- Claude 模型质量高
- 支持所有 Claude 模型

### 2. 使用 OpenAI API

如果你有 OpenAI API Key：

```bash
# 设置 API Key
export OPENAI_API_KEY="your-openai-api-key"

# 运行测试
cargo run --example test_openai_extraction --features llm
```

**优点**：
- 完美支持 function calling
- GPT-4 模型效果优秀
- 已经过充分测试

### 3. 修复 claude-code-api 集成

如果你坚持使用 claude-code-api，需要：

1. **运行自定义 Provider 示例**：
```bash
cargo run --example custom_claude_provider --features llm
```

2. **调试 502 错误**：
```bash
# 确保 Claude CLI 已认证
claude code --version

# 以调试模式运行 claude-code-api
RUST_LOG=debug claude-code-api

# 测试直接 API 调用
cargo run --example test_claude_api_direct
```

### 4. 使用本地 LLM (Ollama)

如果不想使用云服务：

```bash
# 安装 Ollama
brew install ollama

# 下载模型
ollama pull llama2

# 创建本地 provider（需要实现）
```

## 测试示例说明

### 基础结构提取
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ArticleInfo {
    title: String,
    summary: String,
    topics: Vec<String>,
}
```

### 复杂数据提取
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct EcommerceProduct {
    name: String,
    price: Option<f64>,
    features: Vec<String>,
    specifications: HashMap<String, String>,
}
```

## 故障排除

### claude-code-api 502 错误
- 检查 Claude CLI 是否已认证：`claude code --version`
- 确保 claude-code-api 正在运行
- 查看 claude-code-api 日志

### "No function call in response" 错误
- Claude 返回纯文本而非 function call 格式
- 使用自定义 provider 或切换到其他 LLM

### 超时错误
- 增加 claude-code-api 超时设置
- 减少 max_content_length
- 使用更快的模型（如 Haiku）

## 最佳实践

1. **选择合适的模型**：
   - 简单提取：使用 Haiku 或 GPT-3.5
   - 复杂分析：使用 Sonnet 或 GPT-4

2. **优化 Schema**：
   - 保持字段简洁明确
   - 使用可选字段避免提取失败
   - 提供清晰的字段描述

3. **内容预处理**：
   - 使用 `ContentFormat::Text` 减少 token 使用
   - 限制 `max_content_length` 提高速度
   - 启用 `clean_html` 去除无关内容

## 快速开始

最简单的测试方式：

```bash
# 如果有 OpenAI Key
OPENAI_API_KEY=sk-xxx cargo run --example test_openai_extraction --features llm

# 如果有 Anthropic Key  
ANTHROPIC_API_KEY=sk-ant-xxx cargo run --example test_anthropic_direct --features llm
```

这样可以立即看到结构化数据提取的效果！