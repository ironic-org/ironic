# Blog API — v1.0.4 Feature Implementation

An example demonstrating Ironic framework features for building production REST APIs.

## Quick start

```bash
cd examples/blog-api
cargo run
```

Login (demo credentials: `admin` / `ironic`):

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"ironic"}'
```

Use the token:

```bash
TOKEN="<access_token from login>"
curl http://localhost:3000/api/blogs \
  -H "Authorization: Bearer $TOKEN"
```

## All ironic Features

| # | Feature | Status | Where / Why Not |
|---|---------|--------|-----------------|
| 1 | `security` | ✅ | Global middleware: `SecurityHeadersMiddleware`, `RateLimitMiddleware`, `CorsMiddleware` |
| 2 | `compression` | ✅ | `.compression()` on `AxumAdapter` — gzip/brotli/zstd |
| 3 | `metrics` | ✅ | `MetricsLayer` via `.configure_router()` |
| 4 | `validation` | ✅ | `#[derive(garde::Validate)]` on `CreateBlogDto` |
| 5 | `cache` | ✅ | `#[cache(ttl_secs = N)]` on read-heavy GET routes |
| 6 | `logging` | ✅ | `RequestLogging` + `ironic::logging::log::info!` |
| 7 | `scheduling` | ✅ | `StatsReporter` runs via `interval()` or `cron()` |
| 8 | `cron` | ✅ | `cron("0 * * * * *", ...)` in `StatsReporter` |
| 9 | `openapi` | ✅ | `#[api(summary, tag)]` on all blog routes |
| 10 | `redis` | ❌ | InMemory used; Redis setup shown in comments |
| 11 | `hot-reload` | ❌ | `ConfigurationLoader::watch()` not used — `.env` only |
| 12 | `database` | ❌ | In-memory repositories — add SQLx/SeaORM/Diesel for DB |
| 13 | `sqlx` | ❌ | No database integration |
| 14 | `seaorm` | ❌ | No database integration |
| 15 | `diesel` | ❌ | No database integration |
| 16 | `mongodb` | ❌ | No database integration |
| 17 | `auth` | ❌ | Direct JWT via `jsonwebtoken` — not ironic's `auth` module |
| 18 | `jwt` | ❌ | Direct `jsonwebtoken` usage instead |
| 19 | `oauth` | ❌ | No third-party auth integration |
| 20 | `sessions` | ❌ | Stateless JWT API |
| 21 | `authentication` | ❌ | Bundled `auth`+`jwt`+`oauth`+`sessions` — not needed |
| 22 | `events` | ❌ | No pub/sub use case |
| 23 | `realtime` | ❌ | No WebSocket or SSE endpoints |
| 24 | `application-services` | ❌ | Individually enabled instead |
| 25 | `queues` | ❌ | No RabbitMQ/Kafka integration |
| 26 | `microservices` | ❌ | Single-service example |
| 27 | `cqrs` | ❌ | Not applicable |
| 28 | `sagas` | ❌ | Not applicable |
| 29 | `grpc` | ❌ | HTTP REST only |
| 30 | `graphql` | ❌ | REST only |
| 31 | `distributed` | ❌ | Single-service, no distributed patterns |
| 32 | `plugins` | ❌ | No plugin system needed |
| 33 | `devtools` | ❌ | Not configured |
| 34 | `plugin-ecosystem` | ❌ | Not needed |
| 35 | `resilience` | ❌ | No circuit breaker / retry logic |
| 36 | `resilience-ext` | ❌ | No backpressure / bulkhead |
| 37 | `telemetry` | ❌ | OpenTelemetry not configured |
| 38 | `security-cors` | ✅ | Via `security` bundle |
| 39 | `security-rate-limit` | ✅ | Via `security` bundle |
| 40 | `security-headers` | ✅ | Via `security` bundle |
| 41 | `security-csrf` | ❌ | Not needed for JWT API |
| 42 | `static-files` | ❌ | API-only, no static file serving |
| 43 | `multipart` | ❌ | Text-only blog, no file uploads |
| 44 | `uuid` | ❌ | Used directly via `uuid = { workspace = true }` |
| 45 | `versioning` | ❌ | Not configured |
| 46 | `serialization` | ❌ | Not needed |
| 47 | `custom-decorators` | ✅ | `#[decorator(Pagination)]` with `Pagination` extractor |
| 48 | `backtrace` | ❌ | Not needed |
| 49 | `transport-redis` | ❌ | No message transport |
| 50 | `transport-rabbitmq` | ❌ | No message transport |
| 51 | `transport-kafka` | ❌ | No message transport |

## All Lifecycle Hooks

| # | Hook | Phase | Status | Where / Why Not |
|---|------|-------|--------|-----------------|
| 1 | `OnModuleConfigure` | Startup | ❌ | No module-level config validation needed |
| 2 | `OnModuleInit` | Startup | ✅ | `BlogService` — seeds 2 blog posts + 3 categories |
| 3 | `OnApplicationBootstrap` | Startup | ✅ | `StatsReporter` — starts hourly cron task |
| 4 | `OnServerReady` | Startup | ❌ | No readiness probe or health check on startup |
| 5 | `OnRequestInit` | Request | ❌ | No per-request state to initialize |
| 6 | `OnRequestDestroy` | Request | ❌ | No per-request cleanup needed |
| 7 | `OnError` | Runtime | ❌ | Error handling done via `HttpError` directly |
| 8 | `OnGuardDenied` | Runtime | ❌ | Not configured — would log failed JWT validations |
| 9 | `BeforeShutdown` | Shutdown | ❌ | No connection draining or LB deregistration |
| 10 | `OnApplicationShutdown` | Shutdown | ❌ | No shutdown-phase work needed |
| 11 | `OnModuleDestroy` | Shutdown | ❌ | In-memory only — no connections to close |
| 12 | `AfterShutdown` | Shutdown | ❌ | No final metrics flush or cleanup |
| 13 | `LifecycleFuture` | — | ✅ | Used in `OnModuleInit` and `OnApplicationBootstrap` |

## Key Patterns Demonstrated

| Pattern | Code |
|---------|------|
| **`#[controller]` + `#[routes]`** | 5 controllers with full CRUD |
| **`#[guard]`** | `JwtGuard` — validates Bearer token from header |
| **`#[interceptor]`** | `TimingInterceptor` — logs duration for write endpoints |
| **`#[middleware]`** | `RequestTracing`, `RequestLogging` on `BlogsController` |
| **`#[cache]`** | `ttl_secs = 30/60/120` on GET routes |
| **`#[decorator]`** | `Pagination` — extracts `?page=1&size=20` from query |
| **`#[api]`** | OpenAPI metadata on all blog routes |
| **`.exception_filter()`** | Route-level exception filter demo in `blog_tests.rs` |
| **`OnModuleInit`** | `BlogService` seeds 2 posts + 3 categories at startup |
| **`OnApplicationBootstrap`** | `StatsReporter` starts hourly cron task |
| **`cron()`** | `"0 * * * * *"` — runs every minute |
| **`LifecycleFuture`** | All lifecycle hooks return `Box::pin(async move { ... })` |
| **Module DI** | `StatsModule` imports `BlogsModule`, injects `BlogService` |
