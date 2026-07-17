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

## Implemented Features

| Feature | Where | How |
|---------|-------|-----|
| **Modules** | `AppModule`, `BlogsModule`, `StatsModule`, `TasksModule`, `AuthModule` | `#[derive(Module)]` |
| **Dependency Injection** | All services, controllers, repositories | `#[derive(Injectable)]`, `Arc<T>` |
| **Controllers** | 5 controllers across 4 modules | `#[controller("/prefix")]` + `#[routes]` |
| **HTTP Methods** | Full CRUD | `#[get]`, `#[post]`, `#[put]`, `#[delete]` |
| **JSON Body** | `CreateBlogDto`, `UpdateBlogDto`, `LoginDto` | `#[body]` → `JsonBody<T>` |
| **Path Parameters** | `/:id`, `/:slug` | `#[param]` → `PathParameter<T>` |
| **Query Parameters** | `BlogFilterDto` | `#[query]` → `QueryParameters<T>` |
| **JWT Auth** | `JwtGuard` on `BlogsController` | `#[guard(JwtGuard)]` — validates Bearer token |
| **Login Endpoint** | `POST /api/auth/login`, `POST /api/auth/refresh` | JWT access + refresh tokens via `jsonwebtoken` |
| **Interceptors** | `TimingInterceptor` on write endpoints | `#[interceptor(TimingInterceptor)]` |
| **Exception Filters** | `NotFoundFilter` on controllers | `#[exception(NotFoundFilter)]` — JSON 404 |
| **Middleware (attribute)** | `RequestTracing`, `RequestLogging` | `#[middleware(RequestTracing::new())]` |
| **Middleware (global)** | Security, rate limit, CORS | `.middleware(SecurityHeadersMiddleware)` |
| **Cache** | Read-heavy GET routes | `#[cache(ttl_secs = 30/60/120)]` |
| **Cache Backend** | `InMemoryCache` with comment for Redis | `CacheInterceptor::new(Arc::new(InMemoryCache::new(...)))` |
| **Decorators** | Pagination | `#[decorator(Pagination)]` — `?page=1&size=20` |
| **OpenAPI** | `#[api]` annotations on all routes | `#[api(summary = "...", tag = "...")]` |
| **Cron Scheduling** | Hourly stats via `cron("0 * * * * *", ...)` | In `StatsReporter` via `OnApplicationBootstrap` |
| **Validation** | `CreateBlogDto` | `garde::Validate` |
| **Metrics** | HTTP metrics | `MetricsLayer` via `.configure_router()` |
| **Compression** | Gzip/brotli/zstd | `.compression()` on `AxumAdapter` |
| **Security** | Headers, rate limit, CORS | Global middleware |
| **Lifecycle** | Seed data, cron task | `OnModuleInit`, `OnApplicationBootstrap` |
| **Logging** | Structured request logging | `RequestLogging` + `tracing` events |
| **Request Tracing** | Request IDs + spans | `RequestTracing` on controller |
| **Cross-Module DI** | `StatsModule` → `BlogsModule` | Module imports |
| **Testing** | 9 unit tests | `#[ironic::test]` |

## Not Implemented

| Feature | Why Not |
|---------|---------|
| **Database (SQLx/SeaORM/Diesel/MongoDB)** | In-memory stores for zero-setup demo |
| **OAuth2** | No third-party auth integration |
| **Redis Caching** | Commented example in main.rs — add `redis` dep and uncomment |
| **WebSocket Gateways** | REST API — no real-time needs |  
| **Events (EventBus)** | No pub/sub use case |
| **Queue (RabbitMQ/Kafka)** | No async processing needs |
| **gRPC / GraphQL** | HTTP REST only |
| **Multipart / File Upload** | Text-only blog API |
| **Serialization** | Not needed |
| **Telemetry (OpenTelemetry)** | Not configured |
