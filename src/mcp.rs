//! MCP (Model Context Protocol) server implementation
//!
//! This module provides an MCP server that exposes web search and URL fetch
//! capabilities as tools for AI agents. It implements the MCP specification
//! using JSON-RPC 2.0 over stdio.
//!
//! Reference: https://modelcontextprotocol.io/

use crate::config::load_config;
use crate::error::{Result, SearchError};
use crate::fetch::{ContentFormat, FetchOptions, Fetcher};
use crate::output::SearchResponse;
use crate::providers::{build_registry, SearchOptions};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::time::Instant;

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

// MCP Protocol Types

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP List tools result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// MCP Tool content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP Call tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

// Tool Input Types

/// Input parameters for the web_search tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchInput {
    /// The search query string
    pub query: String,
    /// Number of results to return (default: 10)
    #[serde(default = "default_num_results")]
    pub num_results: Option<usize>,
    /// Preferred search provider (optional)
    #[serde(default)]
    pub provider: Option<String>,
}

fn default_num_results() -> Option<usize> {
    Some(10)
}

/// Input parameters for the fetch_url tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchUrlInput {
    /// The URL to fetch
    pub url: String,
    /// Output format: "text" (default), "html", or "markdown"
    #[serde(default = "default_format")]
    pub format: Option<String>,
    /// Maximum content length in bytes (0 = no limit)
    #[serde(default)]
    pub max_length: Option<usize>,
}

fn default_format() -> Option<String> {
    Some("text".to_string())
}

/// MCP Server for cli-web-search
pub struct McpServer {
    /// Server name
    name: String,
    /// Server version
    version: String,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            name: "cli-web-search".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get the list of available tools
    pub fn list_tools(&self) -> ListToolsResult {
        let web_search_schema = schemars::schema_for!(WebSearchInput);
        let fetch_url_schema = schemars::schema_for!(FetchUrlInput);

        ListToolsResult {
            tools: vec![
                Tool {
                    name: "web_search".to_string(),
                    description: "Search the web using configured search providers. Returns a list of search results with titles, URLs, and snippets.".to_string(),
                    input_schema: serde_json::to_value(web_search_schema).unwrap_or_default(),
                },
                Tool {
                    name: "fetch_url".to_string(),
                    description: "Fetch the content of a web page and convert it to text or markdown. Useful for reading web pages.".to_string(),
                    input_schema: serde_json::to_value(fetch_url_schema).unwrap_or_default(),
                },
            ],
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "initialized" => {
                // Notification, no response needed but we'll send an empty success
                JsonRpcResponse::success(request.id, serde_json::Value::Null)
            }
            "tools/list" => self.handle_list_tools(request.id),
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "ping" => JsonRpcResponse::success(request.id, serde_json::json!({})),
            method => {
                // Unknown method
                JsonRpcResponse::error(request.id, -32601, format!("Method not found: {}", method))
            }
        }
    }

    fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
            },
            server_info: ServerInfo {
                name: self.name.clone(),
                version: self.version.clone(),
            },
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or_default())
    }

    fn handle_list_tools(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let result = self.list_tools();
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or_default())
    }

    async fn handle_call_tool(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string());
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing tool name".to_string());
            }
        };

        let arguments = params.get("arguments").cloned().unwrap_or_default();

        let result = match tool_name {
            "web_search" => self.execute_web_search(arguments).await,
            "fetch_url" => self.execute_fetch_url(arguments).await,
            _ => {
                return JsonRpcResponse::error(id, -32602, format!("Unknown tool: {}", tool_name));
            }
        };

        match result {
            Ok(content) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: content,
                    }],
                    is_error: None,
                };
                JsonRpcResponse::success(id, serde_json::to_value(call_result).unwrap_or_default())
            }
            Err(e) => {
                let call_result = CallToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                };
                JsonRpcResponse::success(id, serde_json::to_value(call_result).unwrap_or_default())
            }
        }
    }

    async fn execute_web_search(&self, arguments: serde_json::Value) -> Result<String> {
        let input: WebSearchInput =
            serde_json::from_value(arguments).map_err(|e| SearchError::Api {
                provider: "mcp".to_string(),
                message: format!("Invalid arguments: {}", e),
            })?;

        // Load configuration
        let config = load_config()?;

        // Build provider registry
        let registry = build_registry(&config);

        // Check if any providers are configured
        if registry.configured_providers().is_empty() {
            return Err(SearchError::NoProvidersConfigured);
        }

        let num_results = input.num_results.unwrap_or(10);

        // Build search options
        let options = SearchOptions::new().with_num_results(num_results);

        // Execute search
        let start = Instant::now();
        let (results, provider_used) = registry
            .search_with_fallback(&input.query, &options, input.provider.as_deref())
            .await?;
        let search_time_ms = start.elapsed().as_millis() as u64;

        // Format results as a readable string
        let response = SearchResponse::new(
            input.query.clone(),
            provider_used.to_string(),
            results,
            search_time_ms,
        );

        // Format as text for the AI
        let mut output = format!(
            "Search results for: \"{}\"\nProvider: {} | Results: {} | Time: {}ms\n\n",
            response.metadata.query,
            response.metadata.provider,
            response.results.len(),
            response.metadata.search_time_ms
        );

        for (i, result) in response.results.iter().enumerate() {
            let snippet = if result.snippet.is_empty() {
                "No description available"
            } else {
                &result.snippet
            };
            output.push_str(&format!(
                "{}. {}\n   URL: {}\n   {}\n\n",
                i + 1,
                result.title,
                result.url,
                snippet
            ));
        }

        Ok(output)
    }

    async fn execute_fetch_url(&self, arguments: serde_json::Value) -> Result<String> {
        let input: FetchUrlInput =
            serde_json::from_value(arguments).map_err(|e| SearchError::Api {
                provider: "mcp".to_string(),
                message: format!("Invalid arguments: {}", e),
            })?;

        // Parse format
        let format_str = input.format.as_deref().unwrap_or("text");
        let content_format = match format_str.to_lowercase().as_str() {
            "html" => ContentFormat::Html,
            "markdown" | "md" => ContentFormat::Markdown,
            _ => ContentFormat::Text,
        };

        // Build fetch options
        let options = FetchOptions::new()
            .with_format(content_format)
            .with_max_length(input.max_length.unwrap_or(0));

        let fetcher = Fetcher::with_options(options);

        // Fetch the URL
        let response = fetcher.fetch(&input.url).await?;

        // Format output with metadata
        let mut output = String::new();

        if let Some(title) = &response.title {
            output.push_str(&format!("Title: {}\n", title));
        }
        output.push_str(&format!("URL: {}\n", response.final_url));
        output.push_str(&format!(
            "Content Length: {} bytes\n",
            response.content_length
        ));
        output.push_str("---\n\n");
        output.push_str(&response.content);

        Ok(output)
    }
}

/// Run the MCP server using stdio transport
pub async fn run_mcp_server() -> Result<()> {
    let server = McpServer::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Read lines from stdin
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                continue;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let error_response =
                    JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                let response_json = serde_json::to_string(&error_response).unwrap_or_default();
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        // Handle the request
        let response = server.handle_request(request).await;

        // Write the response
        let response_json = serde_json::to_string(&response).map_err(|e| SearchError::Api {
            provider: "mcp".to_string(),
            message: format!("Failed to serialize response: {}", e),
        })?;

        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_input_defaults() {
        let json = r#"{"query": "test"}"#;
        let input: WebSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "test");
        assert_eq!(input.num_results, Some(10));
        assert!(input.provider.is_none());
    }

    #[test]
    fn test_web_search_input_with_options() {
        let json = r#"{"query": "rust programming", "num_results": 5, "provider": "brave"}"#;
        let input: WebSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "rust programming");
        assert_eq!(input.num_results, Some(5));
        assert_eq!(input.provider, Some("brave".to_string()));
    }

    #[test]
    fn test_fetch_url_input_defaults() {
        let json = r#"{"url": "https://example.com"}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.url, "https://example.com");
        assert_eq!(input.format, Some("text".to_string()));
        assert_eq!(input.max_length, None);
    }

    #[test]
    fn test_fetch_url_input_with_options() {
        let json = r#"{"url": "https://example.com", "format": "markdown", "max_length": 10000}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.url, "https://example.com");
        assert_eq!(input.format, Some("markdown".to_string()));
        assert_eq!(input.max_length, Some(10000));
    }

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new();
        assert_eq!(server.name, "cli-web-search");
        assert!(!server.version.is_empty());
    }

    #[test]
    fn test_mcp_server_default() {
        let server = McpServer::default();
        assert_eq!(server.name, "cli-web-search");
    }

    #[test]
    fn test_list_tools() {
        let server = McpServer::new();
        let result = server.list_tools();

        assert_eq!(result.tools.len(), 2);

        let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"web_search"));
        assert!(tool_names.contains(&"fetch_url"));
    }

    #[test]
    fn test_list_tools_schemas() {
        let server = McpServer::new();
        let result = server.list_tools();

        for tool in &result.tools {
            // Each tool should have a valid input schema
            assert!(tool.input_schema.is_object());

            // Schema should have properties
            let schema = tool.input_schema.as_object().unwrap();
            assert!(schema.contains_key("properties") || schema.contains_key("$schema"));
        }
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "cli-web-search");
    }

    #[tokio::test]
    async fn test_handle_list_tools() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(3)),
            method: "ping".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(4)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_call_tool_missing_params() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(5)),
            method: "tools/call".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
    }

    #[tokio::test]
    async fn test_handle_call_tool_missing_name() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(6)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({})),
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing tool name"));
    }

    #[tokio::test]
    async fn test_handle_call_tool_unknown_tool() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(7)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "unknown_tool",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown tool"));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(
            Some(serde_json::json!(1)),
            serde_json::json!({"data": "test"}),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            Some(serde_json::json!(1)),
            -32600,
            "Invalid Request".to_string(),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_json_rpc_request_parse() {
        let json = r#"{"jsonrpc": "2.0", "id": 1, "method": "test", "params": {"key": "value"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(serde_json::json!(1)));
        assert_eq!(request.method, "test");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_json_rpc_request_parse_minimal() {
        let json = r#"{"jsonrpc": "2.0", "method": "test"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert!(request.id.is_none());
        assert_eq!(request.method, "test");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_server_capabilities_serialize() {
        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability::default()),
        };

        let json = serde_json::to_value(&capabilities).unwrap();
        assert!(json.get("tools").is_some());
    }

    #[test]
    fn test_initialize_result_serialize() {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
            },
            server_info: ServerInfo {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["protocolVersion"], "2024-11-05");
        assert_eq!(json["serverInfo"]["name"], "test");
    }

    #[test]
    fn test_call_tool_result_serialize() {
        let result = CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Hello, world!".to_string(),
            }],
            is_error: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][0]["text"], "Hello, world!");
        // is_error should be skipped when None
        assert!(!json.as_object().unwrap().contains_key("isError"));
    }

    #[test]
    fn test_call_tool_result_with_error() {
        let result = CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Error: something went wrong".to_string(),
            }],
            is_error: Some(true),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["isError"], true);
    }

    #[test]
    fn test_tool_serialize() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "test_tool");
        assert_eq!(json["description"], "A test tool");
        assert!(json["inputSchema"].is_object());
    }

    #[tokio::test]
    async fn test_handle_initialized_notification() {
        let server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "initialized".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        // initialized is a notification, should return null result
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_web_search_input_null_provider() {
        let json = r#"{"query": "test", "provider": null}"#;
        let input: WebSearchInput = serde_json::from_str(json).unwrap();
        assert!(input.provider.is_none());
    }

    #[test]
    fn test_fetch_url_input_null_format() {
        let json = r#"{"url": "https://example.com", "format": null}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert!(input.format.is_none());
    }

    #[test]
    fn test_fetch_url_input_max_length_zero() {
        let json = r#"{"url": "https://example.com", "max_length": 0}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.max_length, Some(0));
    }
}
