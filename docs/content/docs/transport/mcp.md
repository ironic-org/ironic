---
title: MCP Transport
description: Model Context Protocol — AI agent integration for Ironic applications.
---

# MCP Transport

The Model Context Protocol (MCP) transport allows AI agents to interact with your Ironic application through a standardized JSON-RPC 2.0 interface. AI models can discover and invoke application capabilities as typed tools.

## Feature Flag

Enable the `mcp` feature in your `Cargo.toml`:

```toml
[dependencies]
ironic = { version = "1", features = ["mcp"] }
```

## How It Works

MCP exposes registered tools as a JSON-RPC 2.0 endpoint (`POST /mcp`). An AI agent (or any JSON-RPC client) performs:

1. **Initialize** — Handshake to agree on protocol version and capabilities.
2. **List Tools** — Discover available tools and their input schemas.
3. **Call Tool** — Invoke a tool with typed arguments.

## Defining a Tool

Create an `McpTool` with a name, description, JSON Schema for inputs, and an async handler:

```rust
use ironic::{McpTool, json};
use std::sync::Arc;

let greet_tool = McpTool::new(
    "greet",
    "Greets a user by name",
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "The name to greet"
            }
        },
        "required": ["name"]
    }),
    Arc::new(|params| {
        Box::pin(async move {
            let name = params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("world");
            Ok(json!({ "message": format!("Hello, {name}!") }))
        })
    }),
);
```

## Registering Tools and Mounting

Use `McpRouter` to configure the server and register tools, then pass it to `AxumAdapter`:

```rust
use ironic::{AxumAdapter, McpRouter};

let mcp = McpRouter::new()
    .with_name("My API")
    .register_tool(greet_tool);

let adapter = AxumAdapter::new()
    .mcp(mcp);
```

The MCP endpoint is served at `POST /mcp` by default. You can customize the path:

```rust
McpRouter::new()
    .with_endpoint("/api/mcp")
    .register_tool(my_tool);
```

## Using with an AI Agent

Configure your AI agent to point at the MCP endpoint:

```json
{
  "mcpServers": {
    "my-api": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

The agent will discover all registered tools and their input schemas automatically.

## Supported MCP Methods

| Method                    | Status     |
|---------------------------|------------|
| `initialize`              | Supported  |
| `notifications/initialized` | Supported  |
| `tools/list`              | Supported  |
| `tools/call`              | Supported  |
| `resources/list`          | Supported (no resources) |
| `prompts/list`            | Supported (no prompts)   |

## Example: Full Integration

```rust
use std::sync::Arc;
use ironic::{AxumAdapter, McpRouter, McpTool, json};

let greet = McpTool::new(
    "greet",
    "Greets a user",
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"]
    }),
    Arc::new(|params| {
        Box::pin(async move {
            let name = params["name"].as_str().unwrap_or("world");
            Ok(json!({ "message": format!("Hello, {name}") }))
        })
    }),
);

let mcp = McpRouter::new()
    .with_name("Demo API")
    .register_tool(greet);

let adapter = AxumAdapter::new()
    .mcp(mcp)
    .request_body_limit(1024 * 1024);
```

## Spec Reference

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- Protocol version: `2024-11-05`
- Transport: JSON-RPC 2.0 over HTTP

## Alternatives

If you don't need AI-agent-specific protocol features, the [HTTP API](/docs/transport/http) with structured OpenAPI docs works with standard function-calling patterns in most LLM providers.
