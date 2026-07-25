## Why

Ironic covers 58% of NestJS's documented feature surface at production-ready level, but the remaining gaps are defining — microservices (0%), GraphQL (0%), serverless (0%), and several framework fundamentals. Without these, Ironic is a "NestJS-inspired REST framework" rather than a true NestJS alternative. Filling these gaps makes Ironic competitive for the full range of backend applications NestJS serves.

## What Changes

### Microservices Layer (Build from scratch)
- Redesign symmetric `Transport` trait into `MicroserviceClient`/`MicroserviceServer` duality
- Implement `#[message_handler]` proc-macro (request-response with correlation ID) — analogous to existing `#[event_handler]`
- Extend `#[event_handler]` to work across transport boundaries (not just in-process)
- Wire live Redis, RabbitMQ, and Kafka transport backends
- Add TCP transport
- Add pluggable `Serializer`/`Deserializer` traits
- Add hybrid application mode (HTTP server + microservice client/server in one process)
- Add custom transport strategy trait
- Wire exception filters, pipes, guards, interceptors into transport pipeline
- Service discovery abstraction (static + DNS + Consul)

### GraphQL Deep Integration
- Replace thin `async-graphql` re-export with full framework integration
- `#[graphql_resolver]`, `#[graphql_mutation]`, `#[graphql_subscription]` proc-macros
- Codegen for SDL, model sharing with HTTP DTOs
- Query complexity, plugins, federation support
- CLI codegen for GraphQL resources

### Framework Fundamentals
- `ForwardRef<T>` / `#[forward_ref]` for circular dependency resolution
- Lazy module loading at runtime
- `DiscoveryService` for runtime handler/provider discovery
- **BREAKING**: Module system additions for lazy-loading support

### HTTP & API Enhancements
- `HttpClient` service (reqwest-based, DI-injectable, with outbound resilience)
- Cookie parsing/setting utilities
- Global path prefix (`setGlobalPrefix` equivalent)
- Raw body accessor
- HTTPS/TLS configuration, multiple server support
- `setGlobalPrefix` equivalent

### Security & Resilience
- Distributed rate limiting (Redis-backed, shared counter)

### OpenAPI Enhancements
- Mapped types: `PartialType`, `PickType`, `OmitType` derive macros

### CLI Enhancements
- Library sub-generator (`ironic generate library`)
- Script runner

### Serverless Adapter
- AWS Lambda adapter via `lambda-http` / `axum-lambda`

### Documentation & Testing
- Document all new features in `docs/content/docs/`
- Add `sse.md` to `transport/meta.json`
- Add missing docs pages for events, Redis queue specifics, cache parameter decorators
- Integration tests for all new features (in `tests/` and per-crate tests)

## Capabilities

### New Capabilities
- `microservices-core`: Core microservice abstraction — `MicroserviceClient`, `MicroserviceServer`, pattern-based routing, correlation ID matching
- `message-handler-decorator`: `#[message_handler]` proc-macro for request-response patterns
- `transport-redis-live`: Live Redis pub/sub transport backend with reconnect and retry
- `transport-rabbitmq-live`: Live RabbitMQ transport backend with AMQP connection management
- `transport-kafka-live`: Live Kafka transport backend with consumer group management
- `transport-tcp`: TCP socket transport backend
- `custom-transport-strategy`: Custom transport strategy trait for user-defined transports
- `hybrid-application`: Hybrid application mode (HTTP + microservice in one process)
- `graphql-integration`: Deep GraphQL framework integration with resolver/mutation/subscription decorators
- `circular-dependency`: Forward reference support for circular dependency resolution
- `lazy-loading-modules`: Runtime lazy module loading
- `discovery-service`: Runtime handler/provider discovery service
- `http-client-service`: DI-injectable HTTP client for inter-service communication
- `cookie-utils`: Cookie parsing and setting utilities
- `global-path-prefix`: Global API path prefix configuration
- `raw-body-accessor`: Raw request body access
- `https-multi-server`: HTTPS/TLS and multiple server support
- `distributed-rate-limiting`: Redis-backed distributed rate limiting
- `openapi-mapped-types`: `PartialType`, `PickType`, `OmitType` derive macros
- `cli-library-generator`: Library sub-generator for the CLI
- `cli-script-runner`: Script runner command
- `serverless-aws-lambda`: AWS Lambda deployment adapter
- `distributed-tracing-propagation`: Automatic W3C trace context propagation across transports

### Modified Capabilities
- `event-handler-decorator`: Extend `#[event_handler]` to support cross-process event routing (not just in-process)
- `microservice-transports`: Replace symmetric `Transport` trait with asymmetric `MicroserviceClient`/`MicroserviceServer`; wire live backends
- `resilience-extensions`: Add outbound resilience (retry, circuit breaker for inter-service calls)
- `security-middleware`: Add distributed rate limiting
- `sse-framework-integration`: Add to transport sidebar meta.json; document broadcast-based SSE fully
- `queue-redis-backend`: Document Redis queue specifics (QueueConfig, visibility_timeout, retry tracking)
- `cache-decorators`: Document `#[cache_key]`/`#[cache_ttl]` parameter-level decorators

## Impact

- **Code**: New modules in `crates/ironic-distributed/src/` (transport redesign, live backends, TCP transport), new `crates/ironic-graphql/` crate, changes to `crates/ironic-core/src/` (circular deps, lazy loading, discovery), new `crates/ironic-serverless/` crate
- **APIs**: `Transport` trait replaced by `MicroserviceClient`/`MicroserviceServer` — minimal migration for existing channel transport users
- **Dependencies**: New crates and feature flags, optional `lambda-http`, `async-graphql` upgrades, `rdkafka`, `lapin`/`amqplib` wiring, `reqwest`
- **Feature flags**: New flags `serverless`, `http-client`, `circular-deps`, `lazy-loading`, `discovery`, `graphql-integration`, etc.
- **Docs**: 15+ new doc pages, updates to existing pages, blog posts for architecture
- **Tests**: Integration tests for every new capability; existing tests must continue passing
