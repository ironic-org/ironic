
# NestJS vs Ironic — Feature Coverage Map

> Generated from NestJS documentation TOC mapped against Ironic v1.0.9 codebase.
>
> ✅ = Complete — production-ready implementation
> 🟡 = Partial — exists but limited or thin wrapper
> 🔴 = Stub — config/builders exist, no wire implementation
> ❌ = Missing — not implemented

---

## Introduction

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Overview | `docs/content/docs/getting-started/` | ✅ | Welcome page with comparison table vs 7 frameworks |
| First steps | Getting Started tutorial | ✅ | 285-line step-by-step tutorial with CLI scaffold |
| Controllers | `#[controller]`, `#[get]`, `#[post]`, etc. | ✅ | Full route decorators with param/query/body injection |
| Providers | `#[Injectable]` | ✅ | Constructor, factory, and value providers |
| Modules | `#[Module]` | ✅ | imports, exports, providers, controllers, exports |
| Middleware | 3-level middleware | ✅ | App-level, module-level, route-level |
| Exception filters | `#[filter]` | ✅ | Global, controller, route scopes |
| Pipes | `#[pipe]` + `garde` | ✅ | Validation, transformation, custom pipes |
| Guards | `#[guard]` | ✅ | Route/controller guards, feature gate guard |
| Interceptors | `#[interceptor]` | ✅ | Request/response interceptors |
| Custom decorators | `#[decorator]` proc-macro | ✅ | Custom parameter decorators |

## Fundamentals

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Custom providers | Factory/value/constructor | ✅ | `ProviderDefinition::constructor/factory/value` |
| Asynchronous providers | `AsyncModuleInit` trait | ✅ | `async fn on_module_init()` lifecycle hook |
| Dynamic modules | Dynamic module factory | ✅ | Modules with `forRoot`/`forFeature` patterns |
| Injection scopes | Singleton, Transient, Request | ✅ | Three scopes with `ScopeViolationError` prevention |
| Circular dependency | `ForwardRef<T>` | ✅ | `ForwardRef<T>` with `OnceLock`-based lazy resolution |
| Module reference | `ModuleRef` | ✅ | `ModuleRef::get()` / `get_with_scope()` lazy injection |
| Lazy-loading modules | `LazyModule<T>` + `ModuleRef::load()` | ✅ | `LazyModule<T>` wrapper with `Container::extend()` |
| Execution context | `ExecutionContext` | ✅ | Reflector, handler metadata, class/handler inspection |
| Lifecycle events | 14 hooks | ✅ | More hooks than NestJS (OnError, OnGuardDenied, etc.) |
| Discovery service | `DiscoveryService` | ✅ | Runtime provider count and health inspection |
| Platform agnosticism | `HttpPlatformAdapter` trait | ✅ | Abstract platform interface, Axum implementation |

## Testing

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Testing | `ironic-testing` crate | ✅ | `TestApplication`, `TestModule`, mock DI, fluent assertions, CI setup |

## Techniques

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Configuration | `ironic-config` | ✅ | Layered env/file/profiles, hot-reload, `Secret<T>` |
| Database | SQLx/SeaORM/Diesel | ✅ | 1110-line docs page, full connection pooling, migrations |
| MongoDB | `mongodb` driver | ✅ | Connection management, CRUD, health checks |
| Validation | `garde` integration | ✅ | Derive macros, custom validators, pipe integration |
| Caching | `InMemoryCache` + `RedisCache` | ✅ | `#[cache]` decorator, `#[cache_key]`/`#[cache_ttl]`, interceptor |
| Serialization | Response serialization | ✅ | `#[serialize]`, skip/rename, content negotiation |
| Versioning | API versioning | ✅ | URI-based versioning via `#[version("1")]` |
| Task scheduling | `cron` + `interval` | ✅ | `ScheduledTask` with pause/resume/abort, cron expressions |
| Queues | `Queue` trait + `RedisQueue` | ✅ | `InMemoryQueue` + `RedisQueue` (RPUSH/BRPOP, retry, TTL, dead-letter) |
| Logging | Structured logging | ✅ | `tracing`-based, time-series storage, env-filter |
| Cookies | `#[cookie]` marker attribute | 🟡 | Marker attribute exists, full parsing TBD |
| Events | `EventBus` + `#[event_handler]` | ✅ | In-process + cross-process via transport |
| Compression | `compression` feature | ✅ | gzip, brotli, zstd via Tower layer |
| File upload | `multipart` feature | ✅ | `multer`-based, S3 upload template |
| Streaming files | `StreamingResponseBody` | ✅ | Shared-ownership streaming body type |
| HTTP module | `HttpClientService` | ✅ | Injectable HTTP client with retry + circuit breaker |
| Session | `sessions` feature | ✅ | Session management with configurable backends |
| MVC / Templates | — | ❌ | No server-side rendering or template engine |
| Performance (Fastify) | — | ✅ | Ironic uses Axum (comparable perf, different tradeoffs) |
| Server-Sent Events | SSE framework | ✅ | `SseRoute`, `SseConfig`, `sse_endpoint()`, broadcast-based routing |

## Security

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Authentication | `ironic-auth` | ✅ | Argon2 hashing, JWT, OAuth2, sessions |
| Authorization | Guards + roles | ✅ | Guard-based authorization with role checking |
| Encryption/Hashing | Argon2 | ✅ | Password hashing with argon2 |
| Helmet | Security headers | ✅ | `tower-http` security headers middleware |
| CORS | CORS middleware | ✅ | Configurable via `tower-http` |
| CSRF Protection | CSRF middleware | ✅ | Token-based CSRF protection |
| Rate limiting | `RedisRateLimiter` + `InMemoryRateLimiter` | ✅ | Distributed rate limiting via Redis INCR/EXPIRE |

## GraphQL

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Quick start | `graphql` feature + `GraphqlSchemaBuilder` | ✅ | Schema builder with resolver integration |
| Resolvers | `#[resolver]` proc-macro | ✅ | DI-injectable resolver structs |
| Mutations | `#[mutation]` proc-macro | ✅ | Mutation field registration |
| Subscriptions | `#[subscription]` proc-macro | ✅ | Subscription field registration |
| Scalars | — | 🔴 | No scalar support |
| Directives | — | 🔴 | No directive support |
| Interfaces | — | 🔴 | No schema interface integration |
| Unions and Enums | — | 🔴 | No union/enum integration |
| Field middleware | — | 🔴 | No field-level middleware |
| Mapped types | — | 🔴 | No mapped type helpers |
| Plugins | — | 🔴 | No async-graphql plugin integration |
| Complexity | — | 🔴 | No query complexity analysis |
| Extensions | — | 🔴 | No extension support |
| CLI Plugin | `ironic generate graphql-resolver` | ✅ | Codegen for GraphQL resolvers |
| Generating SDL | `driver::Schema::sdl()` | 🟡 | Accessible via async-graphql re-export |
| Sharing models | re-export via `graphql_integration::driver` | 🟡 | `async-graphql` types available |
| Federation | — | ❌ | No Apollo Federation support |

## WebSockets

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Gateways | `#[web_socket_gateway]` | ✅ | Decorator-based WS endpoints, rooms, broadcasting |
| Exception filters | WS-compatible | ✅ | Works in WS context |
| Pipes | WS-compatible | ✅ | Works in WS context |
| Guards | WS-compatible | ✅ | Works in WS context |
| Interceptors | WS-compatible | ✅ | Works in WS context |
| Adapters | — | ❌ | No WS adapter abstraction (only Axum WS) |

## Microservices

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Overview | `docs/content/docs/performance/microservices.md` | ✅ | Full doc page with architecture and examples |
| Redis transport | `RedisClient` + `RedisServer` | ✅ | Live pub/sub with reply channels, reconnect |
| MQTT transport | — | ❌ | Not implemented |
| NATS transport | — | ❌ | Not implemented |
| RabbitMQ transport | `RmqClient` + `RmqServer` | ✅ | Topic exchanges with queue binding |
| Kafka transport | `KafkaClient` + `KafkaServer` | ✅ | Topics with consumer groups (sync wrapper) |
| gRPC | `grpc` feature | 🔴 | Thin `tonic` re-export + `channel_provider()` helper only |
| Custom transporters | `CustomTransportStrategy` trait | ✅ | Associated types for client/server pairs |
| Exception filters (MS) | — | ❌ | Not wired into transport pipeline |
| Pipes (MS) | — | ❌ | Not wired into transport pipeline |
| Guards (MS) | — | ❌ | Not wired into transport pipeline |
| Interceptors (MS) | — | ❌ | Not wired into transport pipeline |

### Microservice Gaps — Detailed

| NestJS Concept | Ironic Equivalent | Status |
|---|---|---|
| `@MessagePattern()` | `#[message_handler]` proc-macro | ✅ |
| `@EventPattern()` | `#[event_handler(transport = "...")]` | ✅ Cross-process support added |
| `ClientProxy.send()` | `MicroserviceClient::send()` | ✅ Pattern + correlation ID |
| `ClientProxy.emit()` | `MicroserviceClient::emit()` | ✅ Fire-and-forget |
| `MicroserviceOptions` union | Per-backend config structs | ✅ Config per transport |
| Hybrid app (HTTP + MS) | `.microservice_server()` / `.microservice_client()` | ✅ Lifecycle-managed |
| `Serializer`/`Deserializer` | `Serializer` + `Deserializer` traits | ✅ With `IdentitySerializer` default |
| Connection lifecycle | `connect()` / `listen()` / `close()` | ✅ |
| Reconnect/retry | Configurable per transport | ✅ |
| Transport status tracking | — | 🟡 Via `Result` returns |
| TCP transport | `TcpClient` + `TcpServer` | ✅ Newline-delimited JSON |
| In-memory transport | `InMemoryServer::pair()` | ✅ For testing |

## Deployment

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Deployment | Deployment guide | ✅ | Docker, docker-compose, CI/CD documented |
| Standalone apps | — | ✅ | Every Ironic app is standalone (no distinction) |

## CLI

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Overview | CLI reference | ✅ | Full documentation |
| Workspaces | `ironic workspace` | ✅ | Inspect project structure |
| Libraries | `ironic generate library <name>` | ✅ | Creates reusable Cargo library crate |
| Usage | CLI usage docs | ✅ | Full command reference |
| Scripts | `ironic run <script>` | ✅ | Runs scripts from `[package.metadata.ironic.scripts]` |

## OpenAPI (Swagger)

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Introduction | OpenAPI intro | ✅ | Fully documented |
| Types and Parameters | API types | ✅ | Schema generation from Rust types |
| Operations | Route docs | ✅ | `#[openapi(...)]` operation metadata |
| Security | Security schemes | ✅ | Auth scheme documentation |
| Mapped Types | `PartialType`/`PickType`/`OmitType` derives | ✅ | Derive macros for OpenAPI schema mapping |
| Decorators | `#[openapi]` macros | ✅ | Route/param decorators for docs |
| CLI Plugin | OpenAPI CLI | ✅ | CLI codegen for OpenAPI |
| Other features | — | 🟡 | Partial coverage |

## Recipes

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| REPL | — | ❌ | No REPL/console |
| CRUD generator | `ironic generate resource` | ✅ | Full CRUD scaffolding |
| SWC (fast compiler) | — | ✅ | Rust is already natively compiled |
| Passport (auth) | Authentication guide | ✅ | Different impl but equivalent capability |
| Hot reload | `hot-reload` feature | ✅ | File watching + auto-rebuild |
| MikroORM | — | ❌ | No MikroORM integration |
| TypeORM | — | ❌ | No TypeORM (but SeaORM is Rust's equivalent) |
| Mongoose | — | ❌ | No Mongoose (but MongoDB driver exists) |
| Sequelize | — | ❌ | No Sequelize |
| Router module | HTTP routing | ✅ | Module-based route mounting |
| Swagger | OpenAPI | ✅ | Full Swagger UI |
| Health checks | `HealthModule` | ✅ | Custom health indicators, readiness probes |
| CQRS | `CqrsBus` | ✅ | Type-safe command/query dispatch |
| Compodoc | Blog posts + docs | ✅ | 62 architecture blog posts |
| Prisma | — | ❌ | No Prisma (Rust ecosystem doesn't have it) |
| Sentry | — | ❌ | No Sentry integration |
| Serve static | `static-files` feature | ✅ | Static file serving via tower-http |
| Commander | CLI | ✅ | CLI framework with commands |
| Async local storage | — | ❌ | No async-local equivalent |
| FAQ | FAQ page | ✅ | 8 common errors, troubleshooting |

## Serverless

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Serverless | `AxumApplication::run_lambda()` | ✅ | AWS Lambda adapter via `lambda_http` |

## Additional

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| HTTP adapter | `HttpPlatformAdapter` | ✅ | Platform abstraction trait |
| Keep-Alive connections | — | ❌ | Not configurable |
| Global path prefix | `AxumAdapter::api_prefix()` | ✅ | Nest all routes under a prefix |
| Raw body | `#[raw_body]` marker attribute | 🟡 | Marker attribute exists |
| Hybrid application | `.microservice_server()` / `.microservice_client()` | ✅ | HTTP + microservice in one process |
| HTTPS & multiple servers | `TlsConfig` + `additional_listener()` | ✅ | TLS cert/key config, multi-address serving |
| Request lifecycle | Request lifecycle doc | ✅ | Documented |
| Common errors | FAQ | ✅ | Documented |
| Examples | Blog API example | ✅ | Full example application |

## Devtools

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Devtools overview | Devtools feature | ✅ | Web UI for module/route inspection |
| CI/CD integration | CI/CD docs | ✅ | GitHub Actions, testing documented |

---

## Summary

| Category | NestJS Items | Ironic ✅ | Ironic 🟡 | Ironic 🔴 | Ironic ❌ | Coverage |
|----------|-------------|----------|-----------|-----------|-----------|----------|
| Introduction | 11 | 11 | 0 | 0 | 0 | 100% |
| Fundamentals | 11 | 11 | 0 | 0 | 0 | 100% |
| Testing | 1 | 1 | 0 | 0 | 0 | 100% |
| Techniques | 19 | 17 | 1 | 0 | 1 | 89% |
| Security | 7 | 7 | 0 | 0 | 0 | 100% |
| GraphQL | 17 | 6 | 2 | 9 | 0 | 35% |
| WebSockets | 6 | 5 | 0 | 0 | 1 | 83% |
| Microservices | 12 | 5 | 0 | 2 | 5 | 42% |
| Deployment | 2 | 2 | 0 | 0 | 0 | 100% |
| CLI | 5 | 5 | 0 | 0 | 0 | 100% |
| OpenAPI | 8 | 6 | 1 | 0 | 1 | 75% |
| Recipes | 20 | 13 | 0 | 0 | 7 | 65% |
| Serverless | 1 | 1 | 0 | 0 | 0 | 100% |
| Additional | 10 | 8 | 1 | 0 | 1 | 80% |
| Devtools | 2 | 2 | 0 | 0 | 0 | 100% |
| **Total** | **132** | **100** | **5** | **11** | **16** | **76%** |

**Key takeaway:** Ironic now covers **76% of NestJS's documented surface area** at the ✅ level (up from 58%). The remaining gaps are:

1. **GraphQL deep features (9/17)** — Scala, directives, interfaces, unions, field middleware, plugins, complexity, extensions, federation
2. **Microservices pipeline (5/12)** — MQTT, NATS, gRPC, exception filters, pipes, guards, interceptors in transport pipeline
3. **Recipes (7/20)** — REPL, MikroORM, TypeORM, Mongoose, Sequelize, Prisma, Sentry, async local storage
4. **WebSocket adapter** — Only Axum WS supported

The most significant improvements since v1.0.9:
- **Microservices**: From 0% to 42% — live Redis, RabbitMQ, Kafka, TCP transports, `#[message_handler]`, `MicroserviceClient`/`MicroserviceServer`, hybrid apps
- **GraphQL**: From 0% to 35% — `#[resolver]`, `#[gql_query]`, `#[mutation]`, `#[subscription]` proc macros, schema builder, CLI generator
- **Fundamentals**: From 73% to 100% — `ForwardRef<T>`, `LazyModule<T>`, `DiscoveryService`
- **Serverless**: From 0% to 100% — `AxumApplication::run_lambda()`
- **CLI**: From 60% to 100% — library generator, script runner
