---
title: Transport Overview
description: Ironic's transport layer handles protocol adaptation — HTTP, WebSocket, GraphQL, OpenAPI, and MCP.
---

# Transport Layer

Ironic provides a unified transport abstraction that lets you expose your application over multiple protocols without changing your core logic. The platform adapter pattern decouples route definitions from the server runtime.

## Available Transports

| Transport | Status | Description |
|-----------|--------|-------------|
| [HTTP](/docs/transport/http) | ✅ Available | RESTful HTTP APIs with Axum |
| [WebSocket](/docs/transport/websocket) | ✅ Available | Real-time bidirectional communication |
| [GraphQL](/docs/transport/graphql) | ✅ Available | Query language for APIs |
| [OpenAPI](/docs/transport/openapi) | ✅ Available | API documentation and client generation |
| [MCP](/docs/transport/mcp) | 🚧 Coming Soon | Model Context Protocol for AI agent integration |

## Architecture

```
┌─────────────────────────────────────────┐
│            Application Logic            │
├─────────────────────────────────────────┤
│        Transport-Native Contracts       │
├──────────┬───────┬───────┬──────┬───────┤
│   HTTP   │  WS   │ Graph │Open  │  MCP  │
│          │       │  QL   │ API  │       │
├──────────┴───────┴───────┴──────┴───────┤
│         Platform Adapter (Axum)         │
└─────────────────────────────────────────┘
```

Each transport compiles down to the platform adapter, which handles the actual network I/O. This means you can add transports independently — they all share the same DI container, middleware pipeline, and configuration system.

## Enabling Transports

Transports are enabled via Cargo features:

```toml
[dependencies]
ironic = { version = "1.0", features = [
    "http",          # HTTP transport (always enabled)
    "realtime",      # WebSocket support
    "graphql",       # GraphQL support
    "openapi",       # OpenAPI doc generation
] }
```

## Platform Adapter

The default platform adapter uses Axum. It implements `HttpPlatformAdapter` and `HttpPlatformApplication` to bridge transport-neutral route definitions to a running HTTP server.

```rust
use ironic::AxumAdapter;

let app = Application::create()
    .module::<MyModule>()
    .await;

AxumAdapter::new()
    .build(app.compile())
    .unwrap()
    .listen(([0, 0, 0, 0], 3000).into(), Shutdown::new(async {
        tokio::signal::ctrl_c().await.unwrap();
        ShutdownSignal::Interrupt
    }))
    .await;
```
