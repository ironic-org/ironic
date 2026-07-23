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

### With the `#[mcp_tool]` Macro (Recommended)

Use the `#[mcp_tool]` attribute on an async function — the JSON Schema is inferred from Rust parameter types:

```rust
use ironic::mcp_tool;

#[mcp_tool("greet", description = "Greets a user by name")]
async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {name}!"))
}
```

This generates a `mcp_tool_greet()` function that returns an `McpTool`.
Supports `String`, `bool`, `i32`/`i64`/`u32`/`u64`/`f32`/`f64`, `Vec<T>`, and `Option<T>` parameters.

### Programmatic API

Create an `McpTool` directly with a name, description, JSON Schema, and async handler:

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

### With `#[mcp_tool]` Macro

```rust
use ironic::{mcp_tool, AxumAdapter, McpRouter};

#[mcp_tool("add", description = "Adds two numbers")]
async fn add(a: i32, b: i32) -> Result<i32, String> {
    Ok(a + b)
}

let mcp = McpRouter::new()
    .with_name("Calculator API")
    .register_tool(mcp_tool_add());

let adapter = AxumAdapter::new()
    .mcp(mcp);
```

### Programmatic API

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
