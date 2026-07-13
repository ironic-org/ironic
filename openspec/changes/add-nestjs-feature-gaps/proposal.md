## Why

Ironic's architecture (modules, DI, pipeline, lifecycle) closely mirrors NestJS, but several mature framework features are still missing or exist only as bare contracts. Adding them makes Ironic production-competitive with NestJS as a batteries-included framework choice, while the existing architecture makes integration straightforward.

## What Changes

- **Validation Pipes**: Add `ParseIntPipe`, `ParseUUIDPipe`, `ValidationPipe` with `garde` integration for declarative request validation
- **Exception Filters**: Add `ExceptionFilter` trait with `@Catch`-style selective catching, global filter registration, and customizable error response shapes
- **API Versioning**: Add URI prefix, header (`Accept-Version`), and media type versioning strategies mapped to controller/route groups
- **Response Serialization**: Add field-level `@Exclude`/`@Expose` decorators and a `SerializeInterceptor` for response transformation
- **Security Middleware**: Add first-class CORS, rate limiting, security headers (HSTS, CSP, XFO), and CSRF protection middleware
- **Compression**: Add response compression middleware (gzip, brotli, deflate)
- **WebSocket Gateways**: Add `@WebSocketGateway` and `@SubscribeMessage` decorators with message routing, rooms, and broadcasting
- **Microservice Transports**: Add Redis, RabbitMQ, and Kafka transport adapters for the existing `Transport`/`Queue` traits
- **Cache Decorators**: Add `@CacheKey`/`@CacheTTL`/`CacheInterceptor` with Redis cache backend
- **Cron Scheduling**: Add cron expression support and `@Cron`/`@Interval`/`@Timeout` decorators
- **Dynamic Modules**: Add `forRoot()`/`forRootAsync()`/`register()` patterns, `@Global()` scope, and `ModuleRef`
- **Optional Dependencies**: Add `@Optional` support to `#[derive(Injectable)]`
- **Custom Decorators**: Add `create_param_decorator!` macro for user-defined parameter decorators

## Capabilities

### New Capabilities
- `validation-pipes`: Declarative request validation with pre-built pipes and `garde` integration
- `exception-filters`: Selective exception catching, global filters, customizable error responses
- `api-versioning`: URI, header, and media type versioning strategies for controllers
- `response-serialization`: Field-level exclude/expose control and response transformation interceptor
- `security-middleware`: Built-in CORS, rate limiting, security headers, and CSRF middleware
- `compression`: Response compression via gzip/brotli/deflate middleware
- `websocket-gateways`: Decorator-based WebSocket endpoints with rooms and broadcasting
- `microservice-transports`: Redis, RabbitMQ, and Kafka adapters for microservice communication
- `cache-decorators`: Declarative caching with `@CacheKey`/`@CacheTTL` and Redis backend
- `cron-scheduling`: Cron expression scheduling with `@Cron`/`@Interval`/`@Timeout` decorators
- `dynamic-modules`: `forRoot()`/`forRootAsync()`/`register()` patterns and `@Global()` scope
- `optional-dependencies`: `@Optional` derive macro support for optional provider injection
- `custom-decorators`: User-extensible parameter decorator creation macro

### Modified Capabilities
- `request-pipeline`: Pipes section to cover global pipe registration and pre-built validation pipes
- `dependency-injection`: Optional dependency requirements and dynamic module patterns
- `procedural-macros`: New derive macros and decorator attribute macros

## Impact

- **New crates**: `ironic-security` (security middleware), backend crates for microservice transports
- **Modified crates**: `ironic-http` (pipes, exception filters, versioning), `ironic-di` (optional deps), `ironic-macros` (new derive + attribute macros), `ironic-services` (cache, scheduling), `ironic-distributed` (transports), `ironic-core` (dynamic modules)
- **New feature flags**: `security`, `compression`, `versioning`, `serialization`, `validation`, `cron`, `custom-decorators`
- **New dependencies**: `garde` (validation), `flate2`/`async-compression` (compression), `redis` (cache backend), `cron` (scheduling), `lapin` (RabbitMQ), `rdkafka` (Kafka)
- **Breaking**: Exception filter system changes error response shape (non-breaking if kept optional); API versioning adds new route metadata fields (backward-compatible)
