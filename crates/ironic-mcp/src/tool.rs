use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::types::McpToolDefinition;

/// The return type of an MCP tool handler: a pinned, boxed future
/// resolving to `Ok(value)` or `Err(error_message)`.
pub type McpToolResult =
    Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>;

/// A boxed async function that implements an MCP tool.
pub type McpToolFn = Arc<dyn Fn(serde_json::Value) -> McpToolResult + Send + Sync>;

/// A registered MCP tool with its definition and handler.
pub struct McpTool {
    /// The tool metadata (name, description, input schema).
    pub definition: McpToolDefinition,
    /// The async handler that executes the tool.
    pub handler: McpToolFn,
}

impl McpTool {
    /// Creates a new MCP tool.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        handler: McpToolFn,
    ) -> Self {
        Self {
            definition: McpToolDefinition {
                name: name.into(),
                description: description.into(),
                input_schema,
            },
            handler,
        }
    }

    /// Returns the tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.definition.name
    }
}

/// An internal registry that maps tool names to tool implementations.
pub struct ToolRegistry {
    tools: Vec<McpTool>,
    by_name: HashMap<String, usize>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Creates an empty tool registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    /// Registers a tool in the registry.
    pub fn register(&mut self, tool: McpTool) {
        let name = tool.name().to_string();
        self.tools.push(tool);
        self.by_name.insert(name, self.tools.len() - 1);
    }

    /// Looks up a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&McpTool> {
        self.by_name.get(name).map(|&i| &self.tools[i])
    }

    /// Returns all registered tools.
    #[must_use]
    pub fn all(&self) -> &[McpTool] {
        &self.tools
    }

    /// Returns the definitions of all registered tools.
    #[must_use]
    pub fn definitions(&self) -> Vec<McpToolDefinition> {
        self.tools.iter().map(|t| t.definition.clone()).collect()
    }
}
