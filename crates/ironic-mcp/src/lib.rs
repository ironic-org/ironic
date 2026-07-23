//! MCP (Model Context Protocol) transport for Ironic.
//!
//! Implements JSON-RPC 2.0 over HTTP so AI agents can discover and invoke
//! Ironic controllers as MCP tools.

mod types;

/// Builder for configuring and constructing an MCP server.
pub mod router;
/// MCP server that exposes tools as JSON-RPC 2.0 endpoints.
pub mod server;
/// Tool definitions and handler types for MCP server.
pub mod tool;

pub use router::McpRouter;
pub use server::{McpConfig, McpServer};
pub use tool::McpTool;
pub use types::{
    InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpPromptArgument,
    McpPromptContent, McpPromptDefinition, McpPromptMessage, McpResourceContents,
    McpResourceDefinition, McpServerCapabilities, McpServerInfo, McpToolDefinition,
};
