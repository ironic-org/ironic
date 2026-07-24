
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
| Circular dependency | — | ❌ | No `@forwardRef` / `ForwardRef` equivalent |
| Module reference | `ModuleRef` | ✅ | `ModuleRef::get()` / `get_with_scope()` lazy injection |
| Lazy-loading modules | — | ❌ | No runtime lazy module loading |
| Execution context | `ExecutionContext` | ✅ | Reflector, handler metadata, class/handler inspection |
| Lifecycle events | 14 hooks | ✅ | More hooks than NestJS (OnError, OnGuardDenied, etc.) |
| Discovery service | — | ❌ | No runtime handler/provider discovery |
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
| Cookies | — | ❌ | No cookie parsing/setting support |
| Events | `EventBus` + `#[event_handler]` | ✅ | Typed in-process pub/sub, dead-letter queue, auto-registration |
| Compression | `compression` feature | ✅ | gzip, brotli, zstd via Tower layer |
| File upload | `multipart` feature | ✅ | `multer`-based, S3 upload template |
| Streaming files | `StreamingResponseBody` | ✅ | Shared-ownership streaming body type |
| HTTP module | — | ❌ | No `HttpService` (axios/reqwest wrapper) for inter-service HTTP calls |
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
| Rate limiting | Rate limit middleware | 🟡 | Per-process only, no distributed rate limiting |

## GraphQL

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Quick start | `graphql` feature | 🟡 | `async-graphql` re-export, `schema_provider()` helper |
| Resolvers | — | 🔴 | No resolver codegen — user writes raw `async-graphql` |
| Mutations | — | 🔴 | No mutation decorator |
| Subscriptions | — | 🔴 | No subscription integration |
| Scalars | — | 🔴 | No scalar support |
| Directives | — | 🔴 | No directive support |
| Interfaces | — | 🔴 | No schema interface integration |
| Unions and Enums | — | 🔴 | No union/enum integration |
| Field middleware | — | 🔴 | No field-level middleware |
| Mapped types | — | 🔴 | No mapped type helpers |
| Plugins | — | 🔴 | No async-graphql plugin integration |
| Complexity | — | 🔴 | No query complexity analysis |
| Extensions | — | 🔴 | No extension support |
| CLI Plugin | — | 🔴 | No codegen for GraphQL |
| Generating SDL | — | 🔴 | No SDL generation |
| Sharing models | — | 🔴 | No model sharing between HTTP + GraphQL |
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
| Overview | — | ❌ | No microservice application concept |
| Redis transport | `RedisTransportConfig` | 🔴 | Config struct + builder exist, `send()`/`receive()` return error |
| MQTT transport | — | ❌ | Not implemented |
| NATS transport | — | ❌ | Not implemented |
| RabbitMQ transport | `RabbitMqTransportConfig` | 🔴 | Config struct + builder exist, `send()`/`receive()` return error |
| Kafka transport | `KafkaTransportConfig` | 🔴 | Config struct + builder exist, `send()`/`receive()` return error |
| gRPC | `grpc` feature | 🔴 | Thin `tonic` re-export + `channel_provider()` helper only |
| Custom transporters | — | ❌ | No `CustomTransportStrategy` trait equivalent |
| Exception filters (MS) | — | ❌ | Not wired into transport pipeline |
| Pipes (MS) | — | ❌ | Not wired into transport pipeline |
| Guards (MS) | — | ❌ | Not wired into transport pipeline |
| Interceptors (MS) | — | ❌ | Not wired into transport pipeline |

### Microservice Gaps — Detailed

| NestJS Concept | Ironic Equivalent | What's Missing |
|---|---|---|
| `@MessagePattern()` | — | No request-response pattern decorator |
| `@EventPattern()` | `#[event_handler]` | Exists but only in-process, not on transport |
| `ClientProxy.send()` | `Transport::send()` | Needs async pattern-based API + correlation ID |
| `ClientProxy.emit()` | `Transport::send()` | Needs async dispatch + no reply |
| `MicroserviceOptions` union | — | No unified transport config type |
| Hybrid app (HTTP + MS) | — | Single-role apps only |
| `Serializer`/`Deserializer` | — | Raw `Vec<u8>` in envelopes |
| Connection lifecycle | — | No `listen()`/`close()` on transports |
| Reconnect/retry | — | No retry strategy per transport |
| Transport status tracking | — | No status observable/stream |
| TCP transport | — | Not implemented |

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
| Libraries | — | ❌ | No library sub-generator |
| Usage | CLI usage docs | ✅ | Full command reference |
| Scripts | — | ❌ | No `nest`-style script runner |

## OpenAPI (Swagger)

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| Introduction | OpenAPI intro | ✅ | Fully documented |
| Types and Parameters | API types | ✅ | Schema generation from Rust types |
| Operations | Route docs | ✅ | `#[openapi(...)]` operation metadata |
| Security | Security schemes | ✅ | Auth scheme documentation |
| Mapped Types | — | ❌ | No `PartialType`/`PickType`/`OmitType` helpers |
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
| Serverless | — | ❌ | No Lambda/Cloud Functions integration |

## Additional

| NestJS | Ironic | Status | Notes |
|--------|--------|--------|-------|
| HTTP adapter | `HttpPlatformAdapter` | ✅ | Platform abstraction trait |
| Keep-Alive connections | — | ❌ | Not configurable |
| Global path prefix | — | ❌ | No `setGlobalPrefix` equivalent |
| Raw body | — | ❌ | No raw body accessor |
| Hybrid application | — | ❌ | No HTTP + microservice coexistence |
| HTTPS & multiple servers | — | ❌ | No HTTPS/TLS or multi-server config |
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
| Fundamentals | 11 | 8 | 0 | 0 | 3 | 73% |
| Testing | 1 | 1 | 0 | 0 | 0 | 100% |
| Techniques | 19 | 16 | 0 | 0 | 3 | 84% |
| Security | 7 | 6 | 1 | 0 | 0 | 86% |
| GraphQL | 17 | 0 | 1 | 16 | 0 | 0% |
| WebSockets | 6 | 5 | 0 | 0 | 1 | 83% |
| Microservices | 12 | 0 | 0 | 3 | 9 | 0% |
| Deployment | 2 | 1 | 0 | 0 | 1 | 50% |
| CLI | 5 | 3 | 0 | 0 | 2 | 60% |
| OpenAPI | 8 | 5 | 1 | 0 | 2 | 63% |
| Recipes | 20 | 13 | 0 | 0 | 7 | 65% |
| Serverless | 1 | 0 | 0 | 0 | 1 | 0% |
| Additional | 10 | 5 | 0 | 0 | 5 | 50% |
| Devtools | 2 | 2 | 0 | 0 | 0 | 100% |
| **Total** | **132** | **76** | **3** | **19** | **34** | **58%** |

**Key takeaway:** Ironic covers **58% of NestJS's documented surface area** at the ✅ level. The biggest gaps are:

1. **Microservices (0/12)** — No live transport backends, no message patterns, no hybrid apps
2. **GraphQL (0/17)** — Thin re-export only, no deep framework integration
3. **Serverless (0/1)** — No deployment adapter
4. **Fundamentals gaps (3/11 missing)** — Circular deps, lazy loading, discovery service
5. **Techniques gaps (3/19)** — Cookies, HTTP module, MVC templates

The microservices gap is by far the most significant — it's the defining feature that separates NestJS from being "Express with decorators."
