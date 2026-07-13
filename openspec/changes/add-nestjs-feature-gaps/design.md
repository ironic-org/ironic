## Context

Ironic currently has a well-architected foundation (modules, DI, pipeline, lifecycle, platform adapters) but lacks several features that production frameworks like NestJS provide. The missing features span request processing (validation pipes, exception filters), transport (WebSocket gateways, microservice adapters), infrastructure (caching, scheduling, security middleware), and developer experience (dynamic modules, custom decorators, optional deps).

The existing architecture makes these additions natural extensions:
- **Request pipeline** already supports middleware, guards, interceptors — exception filters and validation pipes slot into the same chain
- **DI container** already supports scopes, factories, async providers — optional deps and dynamic modules extend the provider model
- **Procedural macros** already generate controller/module/injectable metadata — new attribute macros follow the same pattern
- **Feature flags** already gate database backends, auth providers, etc. — new features follow the same convention

## Goals / Non-Goals

**Goals:**
- Implement 13 new capabilities that close the gap with NestJS
- Each capability must be feature-flagged (opt-in)
- Backward-compatible where possible; where breaking (exception filters, dynamic modules), provide a migration path
- All new code must match existing conventions: same patterns, same error handling, same testing approach
- Pre-built in-memory implementations for development/testing alongside production backends

**Non-Goals:**
- Not rewriting existing working code — extend, don't replace
- Not implementing every NestJS microservice transport — Redis, RabbitMQ, Kafka cover 90% of use cases
- Not adding a class-validator dependency — `garde` is lighter and Rust-idiomatic
- Not adding a full ORM/ODM — Ironic already integrates SQLx, SeaORM, Diesel, MongoDB

## Decisions

### D1: Validation via `garde` crate
`garde` is a pure-Rust validation library with derive macros, no proc macros in the validation rules, and good performance. It's the Rust-idiomatic choice over trying to replicate NestJS's class-validator (which relies on TypeScript decorators and reflection).

### D2: Exception filters as a trait, not a decorator
Rust's type system makes trait-based exception filtering more natural than NestJS's decorator approach. `ExceptionFilter<E>` trait with `catch(&self, exception: E, context: &FilterContext) -> Result<FrameworkResponse, FilterError>`. Global filters implement `ExceptionFilter<Box<dyn Error>>`, route-level filters specify concrete error types.

### D3: API versioning via route metadata + adapter
Versioning is implemented as route metadata (e.g., `ControllerDefinition::version("2024-01", VersioningStrategy::Header)`). The platform adapter (Axum) reads this metadata during route compilation to apply the versioning scheme. No runtime overhead per-request — the version check is compiled into the router.

### D4: Security middleware in a new `ironic-security` crate
Security concerns (CORS, rate limiting, security headers, CSRF) are cross-cutting and deserve their own crate. Each middleware is a separate module following the existing `Middleware` trait. Rate limiting uses a sliding window with configurable backend (in-memory for dev, Redis for production).

### D5: Compression as optional Tower layer
Response compression is implemented as a Tower layer registered through the adapter's existing `configure_router` escape hatch, wired as `AxumAdapter::compression(CompressionLevel::Default)`. This avoids modifying the pipeline and follows the existing security middleware pattern.

### D6: WebSocket gateways via proc macros
`#[WebSocketGateway]` on a struct with `#[SubscribeMessage("event")]` handler methods. The macro generates route registration into the Axum adapter's WebSocket upgrade path, similar to how `#[Controller]` registers HTTP routes. Rooms and broadcasting use a `WsRoom`/`WsBroadcast` abstraction over `tokio::sync::broadcast`.

### D7: Microservice transports as feature-flagged adapters
Each transport (Redis, RabbitMQ, Kafka) lives behind its own feature flag (`transport-redis`, `transport-rabbitmq`, `transport-kafka`) implementing the existing `Transport` trait. This keeps the dependency graph clean — users only compile what they use.

### D8: Caching with `@CacheInterceptor` in pipeline
Caching is implemented as a new interceptor type (`CacheInterceptor`) that checks the cache before handler execution and writes to cache after. `@CacheKey` and `@CacheTTL` are attribute macros that attach metadata to route definitions. The Redis backend is a separate adapter implementing the existing `Cache` trait.

### D9: Cron scheduling via `cron` crate
The `cron` crate provides cron expression parsing. `@Cron("0 * * * * *")` registers a scheduled task with the scheduling module. Unlike NestJS's decorator-based approach, Ironic's `@Cron` generates a `ScheduledTask` registration at module compile time, integrated with lifecycle hooks for auto-start/stop.

### D10: Dynamic modules via `ModuleDefinitionBuilder` extensions
`for_root()` is a `ModuleDefinitionBuilder` method that replaces `import()` for configurable modules. `@Global()` is a marker in the module macro that marks all exported providers as globally visible. `ModuleRef` is a new service that provides runtime access to the DI container for lazy resolution.

### D11: Optional dependencies via `#[injectable]` attribute
`#[injectable]` gains an `optional` field: `#[injectable(optional = [Trait1, Trait2])]`. The derive macro generates `Dependency::optional` instead of `Dependency::required` for the listed types. The resolved value is `Option<T>`.

### D12: Custom decorators via `create_param_decorator!` macro
A declarative macro that takes an extraction function and generates a marker attribute + parameter extractor implementation. This follows the same pattern as the built-in `#[body]`, `#[query]`, `#[param]`, `#[header]` attributes.

## Risks / Trade-offs

- **[Risk]** 13 features in one change is large → Each feature is independent and can be implemented incrementally via the task breakdown. Parallel implementation by different contributors is feasible.
- **[Risk]** New dependencies increase audit surface → All new deps are well-maintained, pure-Rust where possible, and feature-flagged to avoid bloat for users who don't need the feature.
- **[Trade-off]** Validation with `garde` vs hand-rolled — `garde` adds ~50KB to compile time but provides declarative validation syntax. The trade-off favors developer experience.
- **[Trade-off]** Exception filters as trait vs decorator — Less magical but more explicit, which aligns with Rust's philosophy. Users write a trait impl instead of applying a decorator.
- **[Risk]** Redis backend for rate limiting and caching adds operational complexity → In-memory backends are the default; Redis is opt-in for production deployments.
- **[Risk]** WebSocket gateway macro complexity → The macro follows exactly the same pattern as the existing `#[Controller]`/`#[routes]` macros. If the controller macro is maintainable, the gateway macro is too.
