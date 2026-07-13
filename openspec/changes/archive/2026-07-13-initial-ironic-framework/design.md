## Context

The Ironic framework implements a modular, type-safe Rust web application framework on top of Axum. The design is grounded in the RustFrame architecture docs and follows a layered architecture: user-facing API → generated metadata → framework kernel → platform adapter → Axum/Tower/Tokio. The project is organized as a Cargo workspace with 9 initial crates, each with distinct responsibilities and strict dependency rules.

## Goals / Non-Goals

**Goals:**
- Provide a modular application kernel with compile-time module graph validation
- Deliver a type-safe DI container supporting singleton, transient, factory, and value providers with cycle detection
- Implement a deterministic request pipeline: middleware → guards → interceptors → extraction → validation → handler → response
- Build an Axum platform adapter that converts framework routes into Axum routers while exposing Tower escape hatches
- Generate procedural macros that produce calls to public kernel APIs (no independent runtime behavior)
- Provide testing utilities with provider overrides, in-process test apps, and fluent HTTP assertions
- Ship a CLI for scaffolding projects and generating code (module, controller, service, resource)

**Non-Goals:**
- Implement a new async runtime or HTTP stack
- Replace Axum, Tower, or Hyper
- Include GraphQL, queues, CQRS, or microservices in the initial release
- Provide request-scoped DI (deferred)
- Require string-based dependency tokens
- Implement OpenAPI, caching, scheduling, WebSockets, or SSE (deferred)

## Decisions

1. **Synchronous provider construction with separate async factory trait** — Providers use a synchronous `create` method by default. An `AsyncProvider` trait is available for async initialization (e.g., database pools). This keeps the common path simple while supporting async needs.

2. **HashMap<TypeId, ...> for DI container** — Uses `TypeId` as registration keys. Trait-object injection uses explicit wrapper tokens rather than `Any` downcasting on trait objects, avoiding type-safety issues with trait objects.

3. **RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>> for singletons** — Singleton cache uses `RwLock` for concurrency-safe one-time initialization. Arc-wrapped singletons avoid cloning overhead on resolution.

4. **Module compiler produces a validated graph** — Imports are recursively traversed with cycle detection using DFS. Exports are validated against registered providers. The output is a `CompiledApplicationGraph` used at runtime.

5. **Type-erased handlers** — Route handlers are stored as `Arc<dyn ErasedHandler>` to avoid monomorphization in route tables. The erased handler receives a `&dyn Any` controller reference and a `&[Box<dyn Any>]` of extracted parameters.

6. **Platform adapter trait as the boundary** — `HttpPlatformAdapter` defines how framework routes, middleware, and lifecycle map to the HTTP server. The Axum adapter implements this trait, converting `RouteDefinition` into axum routes and resolving controllers through DI.

7. **Purely declarative macros generate calls to public APIs** — Attribute macros (`#[controller]`, `#[get]`, etc.) expand to calls like `register_route(...)` using public types. No hidden state or runtime behavior is introduced by macro expansion.

8. **CLI as an orchestration layer** — The CLI uses `cargo` under the hood for build/test/run. Code generators use `syn` + `quote` + `prettyplease` for syntax-aware source editing. The framework works fully without the CLI.

9. **Testing overrides via a test module builder** — `TestModule` wraps the compiled module graph and allows replacing providers before resolution. No global mutable state. Test applications bind to `127.0.0.1:0` to avoid port conflicts.

## Risks / Trade-offs

- **TypeId-based DI** — Not suitable for generic provider registration (e.g., `register::<Vec<T>>`). Mitigation: encourage explicit wrapper types for generic scenarios.
- **Synchronous Provider::create** — Forces async initialization to use `AsyncProvider` or builder patterns, adding complexity for async setup. Mitigation: document clearly and provide the `AsyncProvider` escape hatch.
- **Erased handler dispatch** — Adds a small dynamic dispatch overhead per request. Mitigation: benchmarks against raw Axum show negligible overhead for typical routes.
- **Macro-generated code readability** — Generated code can be opaque. Mitigation: use `prettyplease` formatting and document the expansion in module-level docs.
- **CLI source editing** — Syntax-aware editing with `syn`/`quote` works for well-formed files but can fail on unusual formatting. Mitigation: fall back to printing manual instructions when safe editing is uncertain.
