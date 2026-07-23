use std::sync::Arc;

use axum::{
    extract::State,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::json;

use super::tool::{McpTool, ToolRegistry};
use super::types::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, McpServerCapabilities, McpServerInfo,
};

/// Configuration for an MCP server.
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// The HTTP path for the MCP endpoint.
    pub endpoint: String,
    /// The server name advertised during initialization.
    pub name: String,
    /// The server version string.
    pub version: String,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            endpoint: "/mcp".into(),
            name: "Ironic MCP Server".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

/// An MCP server that exposes tools as JSON-RPC 2.0 endpoints.
///
/// Use [`McpRouter`](crate::McpRouter) to build a server with registered
/// tools, then call [`into_router`](McpServer::into_router) to get an
/// Axum [`Router`] that can be merged into your application.
pub struct McpServer {
    name: String,
    version: String,
    tools: ToolRegistry,
}

impl McpServer {
    /// Creates a new `McpServer` with the given name and version.
    #[must_use]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tools: ToolRegistry::new(),
        }
    }

    /// Registers a tool implementation.
    pub fn register_tool(&mut self, tool: McpTool) {
        self.tools.register(tool);
    }

    /// Returns the number of registered tools.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tools.all().len()
    }

    /// Consumes the server and returns an Axum [`Router`] with a `POST /mcp` route.
    pub fn into_router(self) -> Router {
        let shared = Arc::new(self);
        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .with_state(shared)
    }

    fn handle_initialize(&self) -> serde_json::Value {
        let tools_cap = if self.tools.all().is_empty() {
            None
        } else {
            let mut caps = std::collections::HashMap::new();
            caps.insert("listChanged".into(), json!(false));
            Some(caps)
        };

        let result = InitializeResult {
            protocol_version: "2024-11-05".into(),
            capabilities: McpServerCapabilities {
                tools: tools_cap,
                resources: None,
                prompts: None,
            },
            server_info: McpServerInfo {
                name: self.name.clone(),
                version: self.version.clone(),
            },
        };
        serde_json::to_value(result).unwrap_or_default()
    }

    fn handle_tools_list(&self) -> serde_json::Value {
        let definitions = self.tools.definitions();
        json!({ "tools": definitions })
    }

    async fn handle_tools_call(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing tool name".to_string())?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("Tool not found: {name}"))?;

        (tool.handler)(arguments).await
    }

    fn handle_prompts_list() -> serde_json::Value {
        json!({ "prompts": [] })
    }

    fn handle_resources_list() -> serde_json::Value {
        json!({ "resources": [] })
    }
}

async fn handle_mcp_request(
    State(server): State<Arc<McpServer>>,
    body: String,
) -> impl IntoResponse {
    let req: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(JsonRpcResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {e}"),
                )),
            )
                .into_response();
        }
    };

    if req.jsonrpc != "2.0" {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::error(
                req.id.unwrap_or_default(),
                -32600,
                "Invalid JSON-RPC version. Must be 2.0.",
            )),
        )
            .into_response();
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response: Json<JsonRpcResponse> = match req.method.as_str() {
        "initialize" => Json(JsonRpcResponse::success(id, server.handle_initialize())),
        "tools/list" => Json(JsonRpcResponse::success(id, server.handle_tools_list())),
        "tools/call" => {
            match server
                .handle_tools_call(&req.params.unwrap_or_default())
                .await
            {
                Ok(result) => {
                    let content = json!({ "content": [{ "type": "text", "text": result }] });
                    Json(JsonRpcResponse::success(id, content))
                }
                Err(msg) => Json(JsonRpcResponse::error(id, -32603, msg)),
            }
        }
        "notifications/initialized" => {
            return (
                axum::http::StatusCode::OK,
                Json(json!({})),
            )
                .into_response();
        }
        "resources/list" => Json(JsonRpcResponse::success(
            id,
            McpServer::handle_resources_list(),
        )),
        "prompts/list" => Json(JsonRpcResponse::success(
            id,
            McpServer::handle_prompts_list(),
        )),
        _ => Json(JsonRpcResponse::error(
            id,
            -32601,
            format!("Method not found: {}", req.method),
        )),
    };
    response.into_response()
}
