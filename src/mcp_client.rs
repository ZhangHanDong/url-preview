//! MCP (Model Context Protocol) client for browser automation
//!
//! This module provides integration with playwright-mcp server to enable
//! browser-based content fetching and JavaScript rendering.

use crate::PreviewError;
use jsonrpc_core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[cfg(feature = "logging")]
use tracing::{debug, instrument};

/// MCP transport type
#[derive(Clone, Debug)]
pub enum McpTransport {
    /// Standard I/O transport (default)
    Stdio,
    /// HTTP with Server-Sent Events (not implemented yet)
    HttpSse(String),
}

/// MCP client configuration
#[derive(Clone, Debug)]
pub struct McpConfig {
    /// Enable MCP integration
    pub enabled: bool,
    /// Command to start MCP server
    pub server_command: Vec<String>,
    /// Transport type
    pub transport: McpTransport,
    /// Browser operation timeout in seconds
    pub browser_timeout: u64,
    /// Maximum concurrent browser sessions
    pub max_sessions: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_command: vec![
                "npx".to_string(),
                "-y".to_string(),
                "@playwright/mcp@latest".to_string(),
            ],
            transport: McpTransport::Stdio,
            browser_timeout: 30,
            max_sessions: 5,
        }
    }
}

/// Browser usage policy
#[derive(Clone, Debug, PartialEq)]
pub enum BrowserUsagePolicy {
    /// Always use browser
    Always,
    /// Never use browser
    Never,
    /// Automatically detect when browser is needed
    Auto,
}

/// MCP tool definition
#[derive(Debug, Serialize, Deserialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP request
#[derive(Debug, Serialize)]
struct McpRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// MCP response
#[derive(Debug, Deserialize)]
struct McpResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(flatten)]
    result: McpResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum McpResult {
    Success { result: Value },
    Error { error: McpError },
}

#[derive(Debug, Deserialize)]
struct McpError {
    #[allow(dead_code)]
    code: i32,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

/// MCP client for browser automation
pub struct McpClient {
    config: McpConfig,
    process: Arc<Mutex<Option<Child>>>,
    request_id: Arc<Mutex<u64>>,
    tools: Arc<Mutex<HashMap<String, McpTool>>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start the MCP server
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn start(&self) -> Result<(), PreviewError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut process_guard = self.process.lock().await;
        if process_guard.is_some() {
            return Ok(()); // Already started
        }
        
        #[cfg(feature = "logging")]
        debug!("Starting MCP server with command: {:?}", self.config.server_command);
        
        let mut cmd = Command::new(&self.config.server_command[0]);
        cmd.args(&self.config.server_command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to start MCP server: {}", e),
            })?;
        
        // Check if process started successfully
        if let Some(stderr) = child.stderr.take() {
            let _stderr_handle = tokio::spawn(async move {
                let mut stderr_reader = BufReader::new(stderr);
                let mut line = String::new();
                while let Ok(n) = stderr_reader.read_line(&mut line).await {
                    if n == 0 { break; }
                    #[cfg(feature = "logging")]
                    debug!("MCP stderr: {}", line.trim());
                    line.clear();
                }
            });
        }
        
        *process_guard = Some(child);
        
        // Initialize connection and discover tools
        drop(process_guard);
        
        // Wait a bit for the server to start
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        self.initialize().await?;
        
        Ok(())
    }
    
    /// Stop the MCP server
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn stop(&self) -> Result<(), PreviewError> {
        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            #[cfg(feature = "logging")]
            debug!("Stopping MCP server");
            
            let _ = child.kill().await;
        }
        Ok(())
    }
    
    /// Initialize connection and discover tools
    async fn initialize(&self) -> Result<(), PreviewError> {
        // Send initialization request
        let init_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "url-preview",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
            id: self.next_request_id().await,
        };
        
        let _response = self.send_request(init_request).await?;
        
        // Send initialized notification
        let initialized_notification = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
            params: Value::Object(serde_json::Map::new()),
            id: 0, // Notifications don't need ID
        };
        
        // Send notification without waiting for response
        self.send_notification(initialized_notification).await?;
        
        // Discover available tools
        let tools_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: Value::Object(serde_json::Map::new()),
            id: self.next_request_id().await,
        };
        
        let tools_response = self.send_request(tools_request).await?;
        if let McpResult::Success { result } = tools_response.result {
            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                let mut tools_map = self.tools.lock().await;
                for tool in tools {
                    if let Ok(mcp_tool) = serde_json::from_value::<McpTool>(tool.clone()) {
                        tools_map.insert(mcp_tool.name.clone(), mcp_tool);
                    }
                }
                
                #[cfg(feature = "logging")]
                debug!("Discovered {} MCP tools", tools_map.len());
            }
        }
        
        Ok(())
    }
    
    /// Get next request ID
    async fn next_request_id(&self) -> u64 {
        let mut id_guard = self.request_id.lock().await;
        *id_guard += 1;
        *id_guard
    }
    
    /// Send a notification (no response expected)
    async fn send_notification(&self, request: McpRequest) -> Result<(), PreviewError> {
        let mut process_guard = self.process.lock().await;
        let child = process_guard.as_mut()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: "MCP server not started".to_string(),
            })?;
        
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: "No stdin available".to_string(),
            })?;
        
        let request_str = serde_json::to_string(&request)
            .map_err(|e| PreviewError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        #[cfg(feature = "logging")]
        debug!("Sending notification: {}", request_str);
        
        // Send request
        stdin.write_all(request_str.as_bytes()).await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to write to stdin: {}", e),
            })?;
        stdin.write_all(b"\n").await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to write newline: {}", e),
            })?;
        stdin.flush().await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to flush stdin: {}", e),
            })?;
        
        Ok(())
    }
    
    /// Send a request to the MCP server
    async fn send_request(&self, request: McpRequest) -> Result<McpResponse, PreviewError> {
        let mut process_guard = self.process.lock().await;
        let child = process_guard.as_mut()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: "MCP server not started".to_string(),
            })?;
        
        // Get stdin and stdout
        let stdin = child.stdin.as_mut()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: "No stdin available".to_string(),
            })?;
        
        let stdout = child.stdout.as_mut()
            .ok_or_else(|| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: "No stdout available".to_string(),
            })?;
        
        // Serialize request
        let request_str = serde_json::to_string(&request)
            .map_err(|e| PreviewError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        #[cfg(feature = "logging")]
        debug!("Sending request: {}", request_str);
        
        // Send request
        stdin.write_all(request_str.as_bytes()).await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to write to stdin: {}", e),
            })?;
        stdin.write_all(b"\n").await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to write newline: {}", e),
            })?;
        stdin.flush().await
            .map_err(|e| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Failed to flush stdin: {}", e),
            })?;
        
        // Read response with timeout
        let mut reader = BufReader::new(stdout);
        let timeout_duration = Duration::from_secs(self.config.browser_timeout);
        
        let response_line = timeout(timeout_duration, async {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => return Err(PreviewError::ExternalServiceError {
                        service: "MCP".to_string(),
                        message: "MCP server closed connection".to_string(),
                    }),
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            #[cfg(feature = "logging")]
                            debug!("Received response: {}", trimmed);
                            
                            // Try to parse as JSON-RPC response
                            if let Ok(response) = serde_json::from_str::<McpResponse>(trimmed) {
                                // Check if this is the response to our request
                                if response.id == request.id {
                                    return Ok(response);
                                }
                            }
                            // If not our response or not valid JSON-RPC, continue reading
                        }
                    },
                    Err(e) => return Err(PreviewError::ExternalServiceError {
                        service: "MCP".to_string(),
                        message: format!("Failed to read from stdout: {}", e),
                    }),
                }
            }
        }).await
            .map_err(|_| PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: format!("Request timed out after {} seconds", self.config.browser_timeout),
            })??;
        
        Ok(response_line)
    }
    
    /// Navigate to a URL using the browser
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn navigate(&self, url: &str) -> Result<(), PreviewError> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "browser_navigate",
                "arguments": {
                    "url": url
                }
            }),
            id: self.next_request_id().await,
        };
        
        let response = self.send_request(request).await?;
        match response.result {
            McpResult::Success { .. } => {
                // After navigation, we need to capture a snapshot
                self.capture_snapshot().await?;
                Ok(())
            },
            McpResult::Error { error } => Err(PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: error.message,
            }),
        }
    }
    
    /// Capture a snapshot of the current page
    async fn capture_snapshot(&self) -> Result<(), PreviewError> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "browser_snapshot",
                "arguments": {}
            }),
            id: self.next_request_id().await,
        };
        
        let response = self.send_request(request).await?;
        match response.result {
            McpResult::Success { .. } => Ok(()),
            McpResult::Error { error } => Err(PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: error.message,
            }),
        }
    }
    
    /// Take a screenshot of the current page
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn take_screenshot(&self) -> Result<Vec<u8>, PreviewError> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "browser_take_screenshot",
                "arguments": {}
            }),
            id: self.next_request_id().await,
        };
        
        let response = self.send_request(request).await?;
        match response.result {
            McpResult::Success { result } => {
                // Extract base64 screenshot data from the result
                if let Some(content) = result.get("content") {
                    if let Some(content_array) = content.as_array() {
                        for item in content_array {
                            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                                if item_type == "image" {
                                    if let Some(data) = item.get("data").and_then(|d| d.as_str()) {
                                        // Decode base64
                                        return base64::Engine::decode(
                                            &base64::engine::general_purpose::STANDARD,
                                            data
                                        ).map_err(|e| PreviewError::ParseError(
                                            format!("Failed to decode screenshot: {}", e)
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Err(PreviewError::ExtractError("No screenshot data in response".to_string()))
            }
            McpResult::Error { error } => Err(PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: error.message,
            }),
        }
    }
    
    /// Evaluate JavaScript in the browser
    #[cfg_attr(feature = "logging", instrument(skip(self)))]
    pub async fn evaluate(&self, script: &str) -> Result<Value, PreviewError> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "browser_evaluate",
                "arguments": {
                    "function": script
                }
            }),
            id: self.next_request_id().await,
        };
        
        let response = self.send_request(request).await?;
        match response.result {
            McpResult::Success { result } => {
                // Extract the evaluation result
                if let Some(content) = result.get("content") {
                    if let Some(content_array) = content.as_array() {
                        for item in content_array {
                            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                                if item_type == "text" {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        // Try to parse as JSON first
                                        if let Ok(json_value) = serde_json::from_str::<Value>(text) {
                                            return Ok(json_value);
                                        }
                                        // Otherwise return as string
                                        return Ok(Value::String(text.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(result)
            }
            McpResult::Error { error } => Err(PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: error.message,
            }),
        }
    }
    
    /// Get the page content as text
    pub async fn get_page_text(&self) -> Result<String, PreviewError> {
        let script = "() => document.body.innerText";
        let result = self.evaluate(script).await?;
        
        result.as_str()
            .ok_or_else(|| PreviewError::ExtractError("Failed to extract page text".to_string()))
            .map(|s| s.to_string())
    }
    
    /// Get the page HTML
    pub async fn get_page_html(&self) -> Result<String, PreviewError> {
        let script = "() => document.documentElement.outerHTML";
        let result = self.evaluate(script).await?;
        
        result.as_str()
            .ok_or_else(|| PreviewError::ExtractError("Failed to extract page HTML".to_string()))
            .map(|s| s.to_string())
    }
    
    /// Wait for page to load
    pub async fn wait_for_load(&self) -> Result<(), PreviewError> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "browser_wait_for",
                "arguments": {
                    "time": 2
                }
            }),
            id: self.next_request_id().await,
        };
        
        let response = self.send_request(request).await?;
        match response.result {
            McpResult::Success { .. } => Ok(()),
            McpResult::Error { error } => Err(PreviewError::ExternalServiceError {
                service: "MCP".to_string(),
                message: error.message,
            }),
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Clean up the process when the client is dropped
        let process = self.process.clone();
        tokio::spawn(async move {
            let mut process_guard = process.lock().await;
            if let Some(mut child) = process_guard.take() {
                let _ = child.kill().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.browser_timeout, 30);
        assert_eq!(config.max_sessions, 5);
    }
    
    #[test]
    fn test_browser_usage_policy() {
        assert_ne!(BrowserUsagePolicy::Always, BrowserUsagePolicy::Never);
        assert_eq!(BrowserUsagePolicy::Auto, BrowserUsagePolicy::Auto);
    }
}