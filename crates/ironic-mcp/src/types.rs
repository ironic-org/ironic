use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcRequest {
    /// The JSON-RPC version string, always `"2.0"`.
    pub jsonrpc: String,
    /// An identifier established by the client. Omitted for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    /// The name of the method to invoke.
    pub method: String,
    /// Parameters for the method call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 response object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse {
    /// The JSON-RPC version string.
    pub jsonrpc: String,
    /// The matching request identifier.
    pub id: serde_json::Value,
    /// The result of a successful call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// The error produced by a failed call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcError {
    /// A number that indicates the error type.
    pub code: i32,
    /// A short description of the error.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// Creates a successful JSON-RPC response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error JSON-RPC response.
    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Describes an MCP tool that AI agents can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolDefinition {
    /// The tool name, unique per server.
    pub name: String,
    /// A human-readable description of what the tool does.
    pub description: String,
    /// A JSON Schema defining the tool's input parameters.
    pub input_schema: serde_json::Value,
}

/// Describes an MCP resource that AI agents can read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceDefinition {
    /// The URI of the resource.
    pub uri: String,
    /// A human-readable name for the resource.
    pub name: String,
    /// A description of the resource contents.
    pub description: String,
    /// The MIME type of the resource, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// The actual content of an MCP resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceContents {
    /// The URI of the resource.
    pub uri: String,
    /// The MIME type of the content.
    pub mime_type: String,
    /// The text content of the resource.
    pub text: String,
}

/// Describes an MCP prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptDefinition {
    /// The prompt name.
    pub name: String,
    /// A description of the prompt.
    pub description: String,
    /// Arguments accepted by the prompt template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// An argument for an MCP prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptArgument {
    /// The argument name.
    pub name: String,
    /// A description of the argument.
    pub description: String,
    /// Whether the argument is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// A message produced by an MCP prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptMessage {
    /// The role of the message author (e.g. "user", "assistant").
    pub role: String,
    /// The message content.
    pub content: McpPromptContent,
}

/// The content of an MCP prompt message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptContent {
    /// The content type (e.g. "text").
    #[serde(rename = "type")]
    pub content_type: String,
    /// The text content.
    pub text: String,
}

/// Capabilities declared by the MCP server during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerCapabilities {
    /// Whether the server supports tool operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Whether the server supports resource operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Whether the server supports prompt operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Metadata about the MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    /// The server name.
    pub name: String,
    /// The server version.
    pub version: String,
}

/// The result of a successful `initialize` handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// The MCP protocol version.
    pub protocol_version: String,
    /// The server's capabilities.
    pub capabilities: McpServerCapabilities,
    /// Metadata about the server.
    pub server_info: McpServerInfo,
}
