use super::server::McpServer;
use super::McpTool;

/// A builder for configuring and constructing an [`McpServer`].
///
/// Register tools via [`register_tool`](McpRouter::register_tool) and then
/// call [`build`](McpRouter::build) to produce the server.
pub struct McpRouter {
    name: String,
    version: String,
    endpoint: String,
    tools: Vec<McpTool>,
}

impl Default for McpRouter {
    fn default() -> Self {
        Self {
            name: "Ironic MCP Server".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            endpoint: "/mcp".into(),
            tools: Vec::new(),
        }
    }
}

impl McpRouter {
    /// Creates a new `McpRouter` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the server name advertised during the `initialize` handshake.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the server version advertised during the `initialize` handshake.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the HTTP path for the MCP JSON-RPC endpoint.
    ///
    /// Defaults to `/mcp`. A leading `/` is added if missing.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        let ep = endpoint.into();
        self.endpoint = if ep.starts_with('/') {
            ep
        } else {
            format!("/{ep}")
        };
        self
    }

    /// Registers a single tool.
    #[must_use]
    pub fn register_tool(mut self, tool: McpTool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Registers multiple tools at once.
    #[must_use]
    pub fn register_tools(mut self, tools: Vec<McpTool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Consumes the router and produces an [`McpServer`] with all registered tools.
    #[must_use]
    pub fn build(self) -> McpServer {
        let mut server = McpServer::new(&self.name, &self.version);
        for tool in self.tools {
            server.register_tool(tool);
        }
        server
    }

    /// Returns the configured HTTP endpoint path.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}
