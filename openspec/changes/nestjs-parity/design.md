## Context

Ironic currently has a symmetric `Transport` trait (both sides `send`/`receive`), three config-only transport stubs (Redis, RabbitMQ, Kafka), a thin `async-graphql` re-export, and several missing fundamental patterns (circular deps, lazy loading, discovery service). The existing codebase patterns — trait-based abstractions, builder patterns, proc-macro-driven codegen, DI integration — provide the idioms for filling these gaps.

## Goals / Non-Goals

**Goals:**
- Live microservice transport backends (Redis pub/sub, RabbitMQ, Kafka, TCP) with NestJS-equivalent ergonomics
- `#[message_handler]` proc-macro for request-response patterns across transports
- Extend `#[event_handler]` to cross-process transport (currently in-process only)
- Hybrid application (HTTP + microservice in one process)
- Deep GraphQL integration (resolver/mutation/subscription decorators with DI)
- Circular dependency resolution, lazy module loading, discovery service
- HTTP client for inter-service calls with outbound resilience
- Distributed rate limiting, global path prefix, raw body accessor, cookie utils
- OpenAPI mapped types, CLI improvements, serverless adapter
- Documentation and tests for everything
- All features compile independently and together

**Non-Goals:**
- Not replacing the existing `Transport` trait immediately — will deprecate but keep for migration window
- Not implementing every NestJS recipe (e.g., MikroORM, Prisma, Sentry are Rust-ecosystem mismatches)
- Not building a full BFF/gateway with response transformation (simple reverse proxy only)
- Not implementing Apollo Federation — that requires GraphQL federation spec compliance which needs separate design

## Decisions

### 1. Microservice Transport Architecture (Two-Trait System)

**Decision:** Replace symmetric `Transport` trait with asymmetric `MicroserviceClient` + `MicroserviceServer` traits, following NestJS's proven model.

```rust
// New trait split (replaces current symmetric Transport):
#[async_trait]
pub trait MicroserviceClient: Send + Sync {
    /// Connect to the transport broker
    async fn connect(&self) -> Result<(), MicroserviceError>;
    /// Send a message and await response (request-response via correlation ID)
    async fn send<T: Serialize, R: DeserializeOwned>(
        &self, pattern: &str, data: &T
    ) -> Result<R, MicroserviceError>;
    /// Emit an event (fire-and-forget)
    async fn emit<T: Serialize>(
        &self, pattern: &str, data: &T
    ) -> Result<(), MicroserviceError>;
    /// Close the connection
    async fn close(&self) -> Result<(), MicroserviceError>;
    /// Status stream for connection state tracking
    fn status(&self) -> StatusStream;
}

#[async_trait]
pub trait MicroserviceServer: Send + Sync {
    /// Listen for incoming messages
    async fn listen(&self) -> Result<(), MicroserviceError>;
    /// Register a message handler (request-response)
    fn on_message(&self, pattern: &str, handler: MessageHandler);
    /// Register an event handler (fire-and-forget)
    fn on_event(&self, pattern: &str, handler: EventHandler);
    /// Close the server
    async fn close(&self) -> Result<(), MicroserviceError>;
    /// Status stream
    fn status(&self) -> StatusStream;
}

// Backend types implement both:
pub struct RedisClient { ... }  // impl MicroserviceClient
pub struct RedisServer { ... }  // impl MicroserviceServer
```

**Rationale:** NestJS proved this model works across 6+ transport backends. The symmetric `Transport` trait doesn't distinguish roles, can't do request-response (no correlation ID routing), and has no connection lifecycle. A single trait trying to do everything would violate the Interface Segregation Principle.

**Alternatives considered:**
- Keep `Transport` and add Client/Server as marker traits — rejected because `send()`/`receive()` semantics differ fundamentally between client and server
- Single `Transport` with mode enum — rejected because it pushes role-specific behavior into runtime checks

### 2. Pattern-Based Routing

**Decision:** String-based pattern matching with `serde_json` for complex patterns, matching NestJS's approach.

```
Client send("user.create", data) → server handler registered for "user.create"
Client send({"service": "users", "action": "create"}, data) → pattern serialized to JSON string key
```

**Rationale:** String patterns are simple, fast, and cover 90% of use cases. JSON stringification of complex patterns gives the remaining 10% when needed. The same `normalizePattern` → routing map approach NestJS uses.

### 3. Serialization Pluggability

**Decision:** `Serializer`/`Deserializer` traits with JSON default, matching NestJS's `ConsumerSerializer`/`ConsumerDeserializer` pattern.

```rust
#[async_trait]
pub trait Serializer {
    async fn serialize<T: Serialize + ?Sized>(&self, value: &T) -> Result<Vec<u8>, SerializationError>;
}

#[async_trait]
pub trait Deserializer {
    async fn deserialize<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T, SerializationError>;
}
```

**Rationale:** Pluggable serialization is essential for Protobuf (gRPC), MessagePack, and custom wire formats. Defaulting to JSON matches NestJS and avoids breaking existing code.

### 4. Hybrid Application Pattern

**Decision:** Extend `ApplicationBuilder` with `.microservice_server()` / `.microservice_client()` methods, spawning transport listeners alongside the HTTP server.

```rust
Application::builder()
    .module(AppModule::definition())
    .microservice_server(RedisServer::new(config))
    .microservice_client(RedisClient::new(config))
    .build()
```

**Rationale:** This matches NestJS's `app.connectMicroservice()` pattern and reuses Ironic's existing lifecycle hooks for startup/shutdown ordering.

### 5. GraphQL Integration Architecture

**Decision:** New `crates/ironic-graphql/` with proc-macro codegen, separate from the existing `ironic-distributed::graphql` thin wrapper. Follow the same pattern as `#[controller]` → route compilation.

```rust
#[graphql_resolver]
#[module]
struct UserResolver {
    user_service: UserService,  // DI-injected
}

#[graphql_query]
async fn users(&self) -> Vec<User> { ... }

#[graphql_mutation]
async fn create_user(&self, input: CreateUserInput) -> User { ... }
```

This compiles to:
1. An `#[Injectable]` resolver struct
2. `async-graphql` `#[Object]`/`#[Subscription]` impl generation
3. Schema merge in the GraphQL module

**Rationale:** Following Ironic's existing codegen pattern (controller → Axum routes) makes the implementation consistent. A separate crate avoids bloating `ironic-distributed` and allows independent feature flagging.

### 6. Circular Dependency Resolution

**Decision:** `ForwardRef<T>` wrapper using `std::sync::OnceLock` + `Weak` references.

```rust
#[derive(Injectable)]
struct ServiceA {
    b: ForwardRef<ServiceB>,
}

impl ServiceA {
    async fn init(&self) {
        let b = self.b.resolve().await;
        b.do_something().await;
    }
}
```

**Rationale:** Rust's ownership model makes true circular references impossible without ref-counting. `ForwardRef<T>` delays resolution until after the DI container is fully built, using `OnceLock` for thread-safe lazy resolution. This matches NestJS's `@ForwardRef(() => ServiceB)` pattern.

### 7. HTTP Client Service

**Decision:** Wrapper around `reqwest::Client` with outbound resilience layers (Retry, CircuitBreaker).

```rust
#[derive(Injectable)]
struct OrderService {
    http: HttpClientService,
}

impl OrderService {
    async fn get_user(&self, id: &str) -> Result<User, HttpError> {
        self.http.get(&format!("http://users/{id}"))
            .with_retry(3)
            .with_circuit_breaker("user-service")
            .send()
            .await
    }
}
```

**Rationale:** `reqwest` is the de facto Rust HTTP client. Wrapping it with the same resilience patterns that exist for the server layer (Retry, CircuitBreaker from `ironic-resilience`) keeps the framework consistent.

### 8. Distributed Rate Limiting

**Decision:** Use Redis `INCR` + `EXPIRE` with atomic Lua script for window-based rate limiting.

**Rationale:** The existing per-process rate limiter uses in-memory counters. Extending to Redis is the simplest distributed approach. Redis `INCR`/`EXPIRE` with a Lua script gives atomic sliding window semantics without external dependencies.

### 9. Serverless Adapter

**Decision:** Use `tower` Service trait + `lambda-http` adapter, wrapping the compiled Axum router.

**Rationale:** Axum is built on `tower`, and `lambda-http` provides `tower`-to-Lambda adaptation. The platform adapter trait already abstracts the HTTP server — a `LambdaAdapter` implementation is the natural extension.

### 10. Documentation Structure

**Decision:** Each new capability gets:
1. A doc page in the relevant `docs/content/docs/` section
2. A blog post in `docs/content/blog/` for architectural deep-dives
3. Integration tests in `tests/` (or per-crate unit tests)
4. Feature flag documentation in `feature-flags.md`

## Risks / Trade-offs

- **[Risk] Transport backend implementation complexity** — RabbitMQ and Kafka have rich semantics (exchanges, consumer groups, offset management). Wiring all features takes significant effort. → **Mitigation:** Ship Redis transport first (simplest, already have `redis` crate), then RabbitMQ, then Kafka. TCP last.
- **[Risk] Breaking change for existing `Transport` users** — Replacing the `Transport` trait API requires migration. → **Mitigation:** Deprecate old trait in v1.x, remove in v2.0. Provide migration guide. `ChannelTransport` users migrate to `ChannelClient`/`ChannelServer`.
- **[Risk] GraphQL proc-macro complexity** — Generating `async-graphql` derives programmatically is error-prone. → **Mitigation:** Start with resolver + query support only in Phase 1, add mutations/subscriptions in Phase 2.
- **[Risk] Scope creep** — 24+ capabilities is a massive surface area. → **Mitigation:** Organize into 4 implementation phases with clear boundaries. Each phase independently ship-able.
- **[Risk] Test infrastructure** — Transport backends need running Redis/RabbitMQ/Kafka for integration tests. → **Mitigation:** Use `testcontainers` for CI; `ChannelTransport` (in-memory) for unit tests.
- **[Risk] Rust async trait limitations** — `async fn` in traits requires `#[async_trait]` or AFIT (available since Rust 1.75). → **Mitigation:** Use `#[async_trait]` for compatibility (already used throughout Ironic).
