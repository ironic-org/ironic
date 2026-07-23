---
title: MCP Transport
description: Model Context Protocol — AI agent integration for Ironic applications.
---

# MCP Transport (Coming Soon)

The Model Context Protocol (MCP) transport will allow AI agents to interact with your Ironic application in a structured way. MCP provides a standardized interface for AI models to discover and invoke application capabilities.

## Status: 🚧 Under Development

MCP support is planned but not yet implemented. This page will be updated once the feature is available.

## What MCP Will Enable

- **Tool Discovery**: AI agents can discover available API endpoints and their capabilities
- **Structured Invocation**: AI models can call endpoints with typed parameters
- **Context Management**: Maintain conversation state across invocations
- **Resource Access**: AI agents can access application resources through a unified interface

## Planned API

```rust
// Future API — subject to change
#[mcp_tool("get_user")]
async fn get_user(id: String) -> Result<User, McpError> {
    // ...
}
```

## Tracking

- **Feature Flag**: `mcp` (planned)
- **Tracking Issue**: [#MCP](https://github.com/ironic-org/ironic/issues)
- **Spec**: [Model Context Protocol](https://modelcontextprotocol.io/)

## Alternatives

While MCP is under development, you can use the [HTTP API](/docs/transport/http) with structured endpoints that AI agents can already consume via function-calling patterns.
