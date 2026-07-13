## 1. Foundation: New crate scaffolding and shared infrastructure

- [x] 1.1 Create `ironic-security` crate with `Cargo.toml`, `lib.rs`, and feature flags
- [x] 1.2 Add new feature flags to workspace `Cargo.toml`: `security`, `compression`, `versioning`, `serialization`, `validation`, `cron`, `custom-decorators`
- [x] 1.3 Add dependencies to workspace `Cargo.toml`: `garde`, `uuid`, `cron`
- [x] 1.4 Add optional dependencies: `tower-http`, `lapin`, `kafka`
- [x] 1.5 Create `FilterContext` struct and `ExceptionFilter` trait in `ironic-http`
- [x] 1.6 Extend `RouteDefinition` and `ControllerDefinition` with pipe, version, cache, and exception filter metadata

## 2. Validation Pipes

- [x] 2.1 `ParameterPipe` trait already exists in `ironic-http/src/pipeline.rs`
- [x] 2.2 Implement `ParseIntPipe`, `ParseFloatPipe`, `ParseBoolPipe`, `ParseUUIDPipe`
- [x] 2.3 Implement `ValidationPipe` with `garde::Validate` integration
- [x] 2.4 Add global, controller, and parameter pipe registration to `CompiledHttpApplication`
- [x] 2.5 Integrate pipes into the request pipeline execution order (before handler, after extraction)
- [x] 2.6 Write tests: pipe transforms, validation errors, global/controller/parameter scope

## 3. Exception Filters

- [x] 3.1 Implement `ExceptionFilter<E>` trait with `catch()` method
- [x] 3.2 Implement `FilterContext` with request, route metadata, and DI container access
- [x] 3.3 Add filter registry to `CompiledHttpApplication` (global, controller, route scopes)
- [x] 3.4 Integrate exception filter dispatch into pipeline error handling
- [x] 3.5 Add default fallback filter that preserves existing error response shape
- [x] 3.6 Write tests: typed filter catches, scope precedence, unhandled fallback, filter context access

## 4. API Versioning

- [x] 4.1 Add `VersioningStrategy` enum (URI, header, media-type) and version metadata
- [x] 4.2 Add `#[controller(version = "...", strategy = "...")]` attribute macro support
- [x] 4.3 Implement URI prefix versioning in the Axum adapter route compilation
- [x] 4.4 Implement header-based (`Accept-Version`) versioning in the Axum adapter
- [x] 4.5 Implement media type versioning in the Axum adapter
- [x] 4.6 Write tests: URI version routing, header version routing, media type routing, unmatched version 404

## 5. Response Serialization

- [x] 5.1 Design `#[exclude]` and `#[expose(role = "...")]` attribute macro syntax on response DTOs
- [x] 5.2 Implement `SerializeInterceptor` that reads exclude/expose metadata and transforms response
- [x] 5.3 Support conditional exposure based on `AuthContext` / user roles
- [x] 5.4 Register `SerializeInterceptor` as a global pipeline interceptor
- [x] 5.5 Write tests: field exclusion, role-based exposure, interceptor transformation

## 6. Security Middleware

- [x] 6.1 Implement CORS middleware with configurable origins, methods, headers, and preflight handling
- [x] 6.2 Implement rate limiting middleware with sliding window algorithm and in-memory backend
- [x] 6.3 Implement rate limiting Redis backend for production deployments
- [x] 6.4 Implement security headers middleware (HSTS, CSP, X-Content-Type-Options, X-Frame-Options)
- [x] 6.5 Implement CSRF protection middleware with synchronizer token pattern
- [x] 6.6 Wire security middleware into `ironic-security` crate with `AxumAdapter` integration
- [x] 6.7 Write tests: CORS allow/block, rate limit 429, security headers present, CSRF token validation

## 7. Compression Middleware

- [x] 7.1 Implement gzip compression middleware as a Tower layer
- [x] 7.2 Implement brotli and deflate compression support
- [x] 7.3 Add content-type allowlist configuration
- [x] 7.4 Add `AxumAdapter::compression(level)` integration
- [x] 7.5 Write tests: compressed response, uncompressed skipped, content-type filtering

## 8. WebSocket Gateways

- [ ] 8.1 Implement `#[WebSocketGateway(path)]` attribute macro
- [ ] 8.2 Implement `#[SubscribeMessage("event")]` attribute macro for message routing
- [ ] 8.3 Implement WebSocket connection lifecycle (connect, disconnect tracking)
- [ ] 8.4 Implement room join/leave and per-room broadcasting
- [ ] 8.5 Implement `broadcast_all()` and `broadcast_room()` server-side APIs
- [ ] 8.6 Integrate gateway route registration into the platform adapter's WebSocket upgrade path
- [ ] 8.7 Write tests: connection, message routing, room broadcast, unhandled message ignored

## 9. Microservice Transports

- [ ] 9.1 Implement `RedisTransport` adapter for `Transport` trait with pub/sub
- [ ] 9.2 Implement `RabbitMqTransport` adapter for `Transport` trait with queues
- [ ] 9.3 Implement `KafkaTransport` adapter for `Transport` trait with topics
- [ ] 9.4 Add typed builder patterns for each transport adapter configuration
- [ ] 9.5 Add feature flags (`transport-redis`, `transport-rabbitmq`, `transport-kafka`) in `ironic-distributed`
- [ ] 9.6 Write tests: transport send/receive (in-memory integration test patterns)

## 10. Cache Decorators

- [ ] 10.1 Implement `#[cache(ttl_secs = N)]` route attribute macro for cache metadata
- [ ] 10.2 Implement `CacheInterceptor` that checks cache pre-handler and writes post-handler
- [ ] 10.3 Implement `RedisCache` backend implementing the `Cache` trait
- [ ] 10.4 Integrate cache metadata into `RouteDefinition` and `CompiledHttpApplication`
- [ ] 10.5 Write tests: cache hit returns cached response, cache miss invokes handler, TTL expiry

## 11. Cron Scheduling

- [ ] 11.1 Add `cron` crate dependency and implement cron expression parsing for scheduling
- [ ] 11.2 Implement `#[cron("expr")]`, `#[interval(ms)]`, `#[timeout(ms)]` attribute macros
- [ ] 11.3 Integrate scheduled task registration into module compilation and lifecycle hooks
- [ ] 11.4 Implement auto-start on `OnApplicationBootstrap` and graceful stop on `OnApplicationShutdown`
- [ ] 11.5 Write tests: cron execution at expected time, interval periodicity, shutdown cancellation

## 12. Dynamic Modules

- [ ] 12.1 Implement `ModuleDefinitionBuilder::for_root(config)` for static module configuration
- [ ] 12.2 Implement `ModuleDefinitionBuilder::for_root_async(config_future)` for async config
- [ ] 12.3 Implement `register()` pattern for parameterized module imports
- [ ] 12.4 Implement `#[global]` module attribute that exports providers globally
- [ ] 12.5 Implement `ModuleRef` service for runtime DI container access
- [ ] 12.6 Write tests: forRoot configuration, async config resolution, global scope, ModuleRef resolution

## 13. Optional Dependencies

- [ ] 13.1 Extend `#[derive(Injectable)]` proc macro to accept `optional = [Type1, Type2]` attribute
- [ ] 13.2 Generate `Dependency::optional` for listed types instead of `Dependency::required`
- [ ] 13.3 Generate `Option<T>` field type for optional dependencies
- [ ] 13.4 Write tests: optional resolves to Some when provider exists, None when missing, required missing is error

## 14. Custom Decorators

- [x] 14.1 Implement `create_param_decorator!` declarative macro with extraction function parameter
- [x] 14.2 Ensure custom decorators generate valid parameter extraction code for the `#[routes]` macro
- [x] 14.3 Support pipe chaining on custom decorators
- [x] 14.4 Write tests: custom header extractor, custom with pipe validation, integration with `#[routes]`

## 15. Documentation and Final Verification

- [ ] 15.1 Add doc pages for each new capability in `docs/content/docs/`
- [ ] 15.2 Update CHANGELOG.md with all new features
- [ ] 15.3 Run full CI pipeline: `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all-features`
- [ ] 15.4 Run `cargo audit` and `cargo deny check` on new dependencies
- [ ] 15.5 Verify feature flags compile independently (each new feature without default features)
