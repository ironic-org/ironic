---
title: "The platform adapter boundary — how Ironic talks to Axum (and could talk to anything)"
description: "A technical walkthrough of the HttpPlatformAdapter trait, the Axum adapter implementation, and why Ironic's architecture makes it web-server agnostic."
date: "2026-07-15"
author: "Ironic Team"
---

# The platform adapter boundary — how Ironic talks to Axum (and could talk to anything)

Most Rust web frameworks are married to their HTTP server. Actix Web runs on Actix. Axum runs on Axum. If the runtime doesn't suit your deployment target, you're out of luck — or you're rewriting handlers.

Ironic deliberately breaks that coupling. The framework knows nothing about Axum. It knows nothing about Hyper, Actix Web, or any other HTTP runtime. It only knows about a single trait — and Axum is just the first adapter that implements it.

---

## The boundary: two traits, clean separation

Open `crates/ironic-platform/src/lib.rs:43` and you'll see the entire contract:

```rust
pub trait HttpPlatformAdapter: Send + 'static {
    type Application: HttpPlatformApplication<Error = Self::Error>;
    type Error: Error + Send + Sync + 'static;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error>;
}
```

`HttpPlatformApplication` sits on the other side:

```rust
pub trait HttpPlatformApplication: Send + 'static {
    type Error: Error + Send + Sync + 'static;

    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>>;
}
```

That's the whole contract. Two traits. No mention of `axum::Router`, no `Request<Body>`, no `tokio::net::TcpListener`. The adapter receives a compiled, protocol-neutral `CompiledHttpApplication` and must produce anything that can `listen`. That "anything" is opaque to Ironic.

The `CompiledHttpApplication` is the framework's runtime representation: a validated container, a set of compiled routes with handler function pointers, and optional WebSocket gateways. The adapter's job is to translate that into a native router and wire it to a socket.

---

## How AxumAdapter builds the router

The Axum adapter starts minimal:

```rust
pub struct AxumAdapter {
    request_body_limit: usize,
    request_timeout: Duration,
    #[cfg(feature = "compression")]
    enable_compression: bool,
    configure_router: Vec<RouterConfigurator>,
}
```

When `build()` is called at `crates/ironic-platform-axum/src/lib.rs:123`, it iterates over every compiled route from `application.routes()`, converts each one to an Axum route handler, and registers it on a `Router`. The per-route conversion happens in `register_route()` at line 222.

Three things happen per route:

**1. HTTP method translation.** `method_filter()` at line 383 maps Ironic's `HttpMethod` enum (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS) to Axum's `MethodFilter`. Unknown methods return `AxumPlatformError::UnsupportedMethod` — the build fails fast rather than silently dropping routes.

**2. Path parameter syntax conversion.** Ironic uses `:param` syntax (`/users/:id`). Axum uses `{param}` syntax (`/users/{id}`). The `native_path()` function at line 406 splits on `/` and rewrites every `:name` segment to `{name}`. The versioning prefix (e.g., `/v1/`) is already baked into the route's `versioned_path` by the framework — the adapter just uses it verbatim.

**3. The handler closure.** Each route becomes an async closure that wraps the framework's handler execution. The closure destructures path parameters from Axum's `Path<HashMap<String, String>>`, runs API version header and media-type checks (`matches_header_version` and `matches_media_type_version`), applies the `tokio::time::timeout` for per-request deadlines, then calls `execute_route()`.

---

## Request/response conversion at the boundary

`execute_route()` at line 297 is where the type translation happens. An Axum `Request<Body>` arrives. The adapter reads the body into bytes (enforcing `body_limit`), then constructs a protocol-neutral `Request`:

```rust
let request = Request::new(parts.method, parts.uri, parts.headers, body)
    .with_path_parameters(parameters);
let mut context = RequestContext::new(request);
```

`Request` uses `http::Method`, `http::Uri`, and `HeaderMap` — the types from the `http` crate that most Rust HTTP libraries already share. The adapter doesn't invent its own request type; it leans on a common denominator.

After the framework executes the handler, the response flows back the other way. `framework_response()` at line 332 destructures `Response` into status, headers, and body — then maps `FrameworkBody::Empty` to `Body::empty()` and `FrameworkBody::Bytes` to `Body::from(bytes)`. The conversion is zero-copy for bytes, heap-allocated only for the status and header clones.

Handler panics are caught with `AssertUnwindSafe` and `catch_unwind`, converting them into `RF_HTTP_HANDLER_PANICKED` errors rather than crashing the server. Every error path — body too large, timeout, panicked handler, domain error — produces a structured JSON error response through `error_response()`.

---

## Tower layers: compression, body limit, timeout

Three operational concerns are applied as Tower layers rather than per-route middleware:

- **Body limit** is enforced inside `execute_route()` via `axum::body::to_bytes(body, body_limit)`. Exceeding the limit returns a `413 Payload Too Large` response before the framework handler ever runs.

- **Timeout** is applied per-request via `tokio::time::timeout(request_timeout, ...)` wrapping the entire handler execution. If the future doesn't complete in time, a `408 Request Timeout` is returned.

- **Compression** (behind the `compression` feature gate) layers `tower_http::compression::CompressionLayer` on the entire router at line 143. It's added *after* all framework routes but *before* the `configure_router` escape-hatch routes — meaning framework routes get compression, native routes don't automatically get it, and you can re-add it inside your own `configure_router` closure if needed.

---

## The `configure_router()` escape hatch

No adapter can cover every use case. Sometimes you need a raw Axum route — a health check that bypasses Ironic's middleware stack, a WebSocket endpoint with custom handshake logic, or a `/metrics` endpoint from `tower-http`.

`configure_router()` at line 96 accepts a `FnOnce(Router) -> Router` closure and stores it. During `build()`, every registered configurator runs sequentially after all framework routes are registered:

```rust
for configure in self.configure_router {
    router = configure(router);
}
```

This is also how the adapter's tests verify that native routes can coexist with framework routes — the test at line 539 registers a raw `/native` Axum route and confirms it responds alongside the `/users/:id` framework route.

The trade-off: native routes don't get the Ironic middleware stack, DI context, or structured error responses. They're raw Axum. But that's the point — it's an escape hatch, not a second framework.

---

## The `listen()` method: from trait to TCP

Once `build()` produces an `AxumApplication` wrapping an Axum `Router`, the `HttpPlatformApplication::listen()` implementation at line 194 takes over:

1. Binds the `SocketAddr` with `tokio::net::TcpListener::bind`
2. Wires the `Shutdown` future into a `tokio::sync::oneshot` channel
3. Calls `axum::serve(listener, self.router).with_graceful_shutdown(graceful)`
4. Returns the `ShutdownSignal` (Interrupt, Terminate, or Custom) to the caller

The `Application::listen_with_shutdown()` at `crates/ironic-core/src/application.rs:310` is the orchestrator. It delegates to the platform's `listen()`, then runs all shutdown lifecycle hooks in reverse initialization order — `application_shutdown` callbacks followed by `module_destroy` callbacks. Even if serving fails, cleanup still runs.

---

## Why a trait boundary matters

The architecture diagram:

```
┌─────────────────────────────────┐
│         Ironic Core              │
│  Application::listen()  │
│  compiled routes + DI container  │
└──────────────┬──────────────────┘
               │
               │  HttpPlatformAdapter::build(Arc<CompiledHttpApplication>)
               │  HttpPlatformApplication::listen(SocketAddr, Shutdown)
               ▼
┌─────────────────────────────────┐
│    HttpPlatformAdapter trait     │  ← platform crate (no HTTP runtime deps)
└──────────────┬──────────────────┘
               │
               │  impl HttpPlatformAdapter for AxumAdapter
               ▼
┌─────────────────────────────────┐
│         AxumAdapter              │
│  route translation + Tower layers│
│  Axum router construction        │
└──────────────┬──────────────────┘
               │
               │  axum::serve(router).with_graceful_shutdown(...)
               ▼
┌─────────────────────────────────┐
│         Axum Router              │
│  tokio::net::TcpListener         │
└─────────────────────────────────┘
```

`ironic-platform` depends only on `ironic-http` for `CompiledHttpApplication` and `Request`/`Response`. It doesn't import Axum, Hyper, or any HTTP server crate. `ironic-platform-axum` depends on `axum`, `tower`, and `tokio` — but those are adapter-internal details invisible to the framework.

---

## Writing an adapter for Actix Web, Hyper, or a custom server

The recipe is always the same. Implement `HttpPlatformAdapter`:

```rust
impl HttpPlatformAdapter for ActixAdapter {
    type Application = ActixApplication;
    type Error = ActixPlatformError;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error> {
        // 1. Create an Actix App builder
        // 2. For each route in application.routes():
        //    - Convert ironic_http::HttpMethod → actix_web::http::Method
        //    - Convert :param syntax → {param}
        //    - Register a service factory that wraps execute_route()
        // 3. Apply Actix middleware equivalents
        // 4. Return the configured App
    }
}
```

Then implement `HttpPlatformApplication`:

```rust
impl HttpPlatformApplication for ActixApplication {
    type Error = ActixPlatformError;

    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
        Box::pin(async move {
            let server = actix_web::HttpServer::new(move || self.app.clone())
                .bind(address)?;
            // Wire shutdown signal into server.stop()
            server.run().await
        })
    }
}
```

The framework doesn't change. The `Application` struct stores a generic `P: HttpPlatformApplication` at `application.rs:235`. No code in `ironic-core` or `ironic-http` needs to know which HTTP runtime is handling the bytes.

---

## What the trait boundary costs

The adapter layer isn't free. Each route registration allocates a closure. Path syntax conversion does string allocation per segment. The `into_parts()` / `Request::new()` round-trip clones header maps. For Axum, which already uses `http::Method` and `http::HeaderMap` internally, some of this is unavoidable boilerplate. A `From` impl between `axum::http::Request<Body>` and `Request` could reduce it — and that's an optimization the adapter author controls, not the framework.

But the architectural benefit is real. Ironic can target WebAssembly servers via `wasm-bindgen`-based adapters. It can target embedded devices with minimal HTTP stacks. It can be tested without binding a TCP port — the tests at lines 496-503 use `oneshot()` on the raw Axum router, never touching a socket. The trait boundary makes all of this possible without a single `#[cfg]` in core.
