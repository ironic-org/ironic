## 1. Framework Fundamentals

- [ ] 1.1 Implement `ForwardRef<T>` wrapper using `std::sync::OnceLock` for lazy circular dependency resolution
- [ ] 1.2 Add `#[forward_ref]` attribute macro for marking constructor parameters as forward references
- [ ] 1.3 Update DI container to detect and resolve circular dependencies via `ForwardRef`
- [ ] 1.4 Implement lazy module loading with `LazyModule<T>` wrapper and `ModuleRef::load()` method
- [ ] 1.5 Add lifecycle hook integration for lazy modules (execute OnModuleInit on load)
- [ ] 1.6 Implement `DiscoveryService` for runtime provider/handler/module inspection
- [ ] 1.7 Add `global_path_prefix` to `ApplicationBuilder` and apply to all routes in `AxumAdapter`
- [ ] 1.8 Add `#[raw_body]` parameter decorator for accessing raw request body bytes
- [ ] 1.9 Add cookie parsing (`#[cookie]` decorator) and setting utilities
- [ ] 1.10 Add HTTPS/TLS configuration to `AxumAdapter` with certificate and key paths
- [ ] 1.11 Add multi-server support (HTTP + HTTPS simultaneously) to `AxumAdapter`
- [ ] 1.12 Implement `PartialType`, `PickType`, `OmitType` derive macros for OpenAPI schema mapping

## 2. Microservices Core Architecture

- [ ] 2.1 Define `MicroserviceClient` trait with `send()`, `emit()`, `connect()`, `close()`, `status()` methods
- [ ] 2.2 Define `MicroserviceServer` trait with `listen()`, `close()`, `on_message()`, `on_event()`, `status()` methods
- [ ] 2.3 Implement `InMemoryClient` and `InMemoryServer` (replacing `ChannelTransport`) with the new traits
- [ ] 2.4 Add correlation ID generation and routing map for request-response matching
- [ ] 2.5 Implement `Serializer` and `Deserializer` traits with default JSON implementation
- [ ] 2.6 Add `IdentitySerializer` and `IncomingRequestDeserializer` matching NestJS patterns
- [ ] 2.7 Implement `MicroserviceError` and `MicroserviceResult` types
- [ ] 2.8 Add `#[message_handler]` proc-macro for request-response pattern registration
- [ ] 2.9 Add hybrid application support (`.microservice_server()` / `.microservice_client()` on `ApplicationBuilder`)
- [ ] 2.10 Implement `CustomTransportStrategy` trait for user-defined transport backends
- [ ] 2.11 Add pattern normalization and routing infrastructure (string + JSON complex patterns)
- [ ] 2.12 Implement W3C trace context propagation (traceparent/tracestate) through transport envelopes
- [ ] 2.13 Deprecate old `Transport` trait with migration warning

## 3. Transport Backends

- [ ] 3.1 Implement live `RedisClient` and `RedisServer` using `redis` crate pub/sub with reconnect logic
- [ ] 3.2 Add Redis transport configuration (host, port, retry_attempts, retry_delay, wildcards, serializer/deserializer)
- [ ] 3.3 Implement live `RmqClient` and `RmqServer` using `lapin` crate with exchange/queue binding
- [ ] 3.4 Add RabbitMQ transport configuration (urls, exchange, queue, prefetch_count, serializer/deserializer)
- [ ] 3.5 Implement live `KafkaClient` and `KafkaServer` with consumer group management
- [ ] 3.6 Add Kafka transport configuration (brokers, topic, consumer_group, serializer/deserializer, producer_only_mode)
- [ ] 3.7 Implement `TcpClient` and `TcpServer` using `tokio::net::TcpStream`/`TcpListener`
- [ ] 3.8 Add TCP transport configuration (host, port, tls_options, max_buffer_size)
- [ ] 3.9 Update existing `ChannelTransport` to implement both `MicroserviceClient` and `MicroserviceServer`

## 4. Event Handler Cross-Process Support

- [ ] 4.1 Extend `#[event_handler]` proc-macro to accept `transport = "redis"` parameter
- [ ] 4.2 Add `on_event()` registration on `MicroserviceServer` when transport is specified
- [ ] 4.3 Preserve existing in-process `EventBus` behavior when no transport is specified
- [ ] 4.4 Add integration test: event published on one process, received by handler in another via Redis

## 5. GraphQL Deep Integration

- [ ] 5.1 Create `crates/ironic-graphql/` crate with `graphql-integration` feature flag
- [ ] 5.2 Implement `#[graphql_resolver]` proc-macro generating `async-graphql` `#[Object]` impl with DI injection
- [ ] 5.3 Implement `#[graphql_query]` proc-macro for query field registration
- [ ] 5.4 Implement `#[graphql_mutation]` proc-macro for mutation field registration
- [ ] 5.5 Implement `#[graphql_subscription]` proc-macro for subscription field registration
- [ ] 5.6 Add GraphQL module integration (auto-merge resolvers from imported modules into Schema)
- [ ] 5.7 Add model sharing between HTTP DTOs and GraphQL types
- [ ] 5.8 Implement GraphQL playground/SDL endpoint codegen
- [ ] 5.9 Add GraphQL CLI generator (`ironic generate graphql-resolver`)

## 6. HTTP Client & Outbound Resilience

- [ ] 6.1 Create `HttpClientService` as injectable wrapper around `reqwest::Client`
- [ ] 6.2 Add `get<T>`, `post<T, R>`, `put<T, R>`, `delete<T>` typed methods
- [ ] 6.3 Add service URI scheme resolution (`service://service-name/path` → resolved via service registry)
- [ ] 6.4 Implement outbound retry middleware for `HttpClientService` with exponential backoff
- [ ] 6.5 Implement outbound circuit breaker middleware for `HttpClientService`
- [ ] 6.6 Add W3C trace context auto-injection to outbound HTTP requests

## 7. Distributed Security

- [ ] 7.1 Implement `RateLimitBackend` trait with `InMemoryRateLimiter` and `RedisRateLimiter`
- [ ] 7.2 Implement Redis-based sliding window rate limiter using Lua script (INCR + EXPIRE)
- [ ] 7.3 Update rate limit middleware to accept configurable backend
- [ ] 7.4 Add documentation for distributed rate limiting configuration

## 8. CLI Enhancements

- [ ] 8.1 Implement `ironic generate library <name>` sub-generator with crate scaffold and `#[Module]`
- [ ] 8.2 Implement `ironic run <script>` command for running project-defined scripts
- [ ] 8.3 Add script definition to `Cargo.toml` metadata parsing

## 9. Serverless Adapter

- [ ] 9.1 Add `lambda-http` dependency behind `serverless` feature flag
- [ ] 9.2 Implement `LambdaAdapter` using `HttpPlatformAdapter` trait
- [ ] 9.3 Add Lambda handler creation function that wraps compiled Ironic application

## 10. Documentation

- [ ] 10.1 Write microservices overview doc page (`docs/content/docs/performance/microservices.md`)
- [ ] 10.2 Write `#[message_handler]` decorator doc page with request-response examples
- [ ] 10.3 Write transport backends doc page (Redis, RabbitMQ, Kafka, TCP configuration)
- [ ] 10.4 Write hybrid application doc page
- [ ] 10.5 Write GraphQL integration doc page
- [ ] 10.6 Write `HttpClientService` doc page with inter-service HTTP examples
- [ ] 10.7 Write distributed tracing doc page
- [ ] 10.8 Write serverless (Lambda) deployment doc page
- [ ] 10.9 Write distributed rate limiting doc page
- [ ] 10.10 Write circular dependency resolution doc page
- [ ] 10.11 Write discovery service doc page
- [ ] 10.12 Add missing SSE doc page to `docs/content/docs/transport/meta.json`
- [ ] 10.13 Add Redis queue specifics doc (`QueueConfig`, `visibility_timeout`, retry tracking)
- [ ] 10.14 Add `#[cache_key]` / `#[cache_ttl]` usage documentation to cache-decorators page
- [ ] 10.15 Add `EventBus` and `#[event_handler]` documentation page
- [ ] 10.16 Add OpenAPI mapped types documentation
- [ ] 10.17 Update feature-flags.md with all new feature flags
- [ ] 10.18 Write architecture blog posts for key design decisions (microservices redesign, GraphQL integration)

## 11. Testing & Verification

- [ ] 11.1 Write unit tests for `ForwardRef<T>` circular dependency resolution
- [ ] 11.2 Write unit tests for `DiscoveryService`
- [ ] 11.3 Write unit tests for lazy module loading
- [ ] 11.4 Write unit tests for `MicroserviceClient`/`MicroserviceServer` traits with `InMemoryClient`/`InMemoryServer`
- [ ] 11.5 Write unit tests for `Serializer`/`Deserializer` traits
- [ ] 11.6 Write integration tests for `RedisClient`/`RedisServer` with local Redis (via testcontainers)
- [ ] 11.7 Write integration tests for hybrid application (HTTP + microservice)
- [ ] 11.8 Write integration tests for `#[message_handler]` decorator
- [ ] 11.9 Write integration tests for cross-process event handler
- [ ] 11.10 Write integration tests for `HttpClientService` with retry and circuit breaker
- [ ] 11.11 Write integration tests for distributed rate limiting with Redis
- [ ] 11.12 Write GraphQL resolver/mutation/query integration tests
- [ ] 11.13 Write end-to-end microservices example (two services communicating via Redis transport)
- [ ] 11.14 Verify all features compile independently (feature flag matrix)
- [ ] 11.15 Run full test suite: `cargo test --all-features`
- [ ] 11.16 Run clippy: `cargo clippy --all-features`
