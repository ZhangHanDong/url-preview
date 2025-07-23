//! Direct API test to understand Claude's response format
//!
//! Run: cargo run --example test_claude_api_direct

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Claude API Direct Response\n");
    
    let client = reqwest::Client::new();
    
    // Test 1: Simple completion request
    println!("1️⃣ Testing simple completion:");
    let request = json!({
        "model": "claude-3-5-haiku-20241022",
        "messages": [{
            "role": "user",
            "content": "Extract: title, summary, keywords from: Rust is a systems programming language focused on safety and performance."
        }],
        "max_tokens": 200,
        "temperature": 0.0
    });
    
    test_request(&client, request).await?;
    
    // Test 2: With function calling format (OpenAI style)
    println!("\n2️⃣ Testing with function calling:");
    let request = json!({
        "model": "claude-3-5-haiku-20241022",
        "messages": [{
            "role": "user",
            "content": "Extract article information"
        }],
        "functions": [{
            "name": "extract_article",
            "description": "Extract article information",
            "parameters": {
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "summary": {"type": "string"},
                    "keywords": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["title", "summary", "keywords"]
            }
        }],
        "function_call": "auto",
        "max_tokens": 200,
        "temperature": 0.0
    });
    
    test_request(&client, request).await?;
    
    // Test 3: With tools format (newer OpenAI style)
    println!("\n3️⃣ Testing with tools format:");
    let request = json!({
        "model": "claude-3-5-haiku-20241022",
        "messages": [{
            "role": "user",
            "content": "Extract article data from: Rust programming language"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "extract_data",
                "description": "Extract structured data",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"}
                    }
                }
            }
        }],
        "tool_choice": "auto",
        "max_tokens": 200
    });
    
    test_request(&client, request).await?;
    
    Ok(())
}

async fn test_request(
    client: &reqwest::Client,
    request: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Request: {}", serde_json::to_string(&request)?);
    
    let response = client
        .post("http://localhost:8080/v1/chat/completions")
        .header("Authorization", "Bearer not-needed")
        .json(&request)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    println!("📥 Status: {}", status);
    
    if status.is_success() {
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(json) => {
                println!("✅ Response:");
                if let Some(choices) = json["choices"].as_array() {
                    for (i, choice) in choices.iter().enumerate() {
                        println!("\nChoice {}:", i);
                        
                        // Check different response formats
                        if let Some(content) = choice["message"]["content"].as_str() {
                            println!("  Content: {}", content);
                        }
                        
                        if choice["message"]["function_call"].is_object() {
                            println!("  Function call: {}", 
                                serde_json::to_string(&choice["message"]["function_call"])?);
                        }
                        
                        if choice["message"]["tool_calls"].is_array() {
                            println!("  Tool calls: {}", 
                                serde_json::to_string(&choice["message"]["tool_calls"])?);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ JSON parse error: {}", e);
                println!("Raw: {}", text);
            }
        }
    } else {
        println!("❌ Error: {}", text);
    }
    
    println!("{}", "-".repeat(60));
    Ok(())
}