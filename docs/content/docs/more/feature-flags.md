---
title: Feature Flag Reference
description: Every feature flag in Ironic — what it enables, what it pulls in, and where to learn more.
---

# Feature Flag Reference

Every feature flag in `Cargo.toml` (51 total), organized by category.

## Default
| Flag | What it enables |
|------|----------------|
| `default` | `hot-reload`, `openapi`, `logging` |

## Database
| Flag | Enables | Extra |
|------|---------|-------|
| `database` | Bundled: `sqlx`, `seaorm`, `diesel`, `mongodb`, `redis` | |
| `sqlx` | SQLx async PostgreSQL/MySQL/SQLite | Use `sqlx-postgres`, `sqlx-mysql`, `sqlx-sqlite` for drivers |
| `seaorm` | SeaORM async ORM | Use `seaorm-postgres`, etc. |
| `diesel` | Diesel sync ORM with r2d2 pooling | |
| `mongodb` | MongoDB Rust driver with TLS | |
| `redis` | Redis async client (`aio`, `tokio-comp`, `connection-manager`) | Enables `RedisCache`, `RedisSessionStore`, `RedisRateLimiter` |

## Authentication
| Flag | Enables | Deps |
|------|---------|------|
| `auth` | Password hashing (Argon2id) | `argon2` |
| `jwt` | JWT creation/validation | `jsonwebtoken` |
| `oauth` | OAuth2 helpers (code exchange, state validation) | `oauth2` |
| `sessions` | Session management (in-memory + Redis stores) | `getrandom` |
| `authentication` | Bundled: `auth`, `jwt`, `oauth`, `sessions` | All of the above |

## Application Services
| Flag | Enables |
|------|---------|
| `cache` | `Cache` trait, `InMemoryCache`, `RedisCache` (with `redis`), `CacheInterceptor` |
| `scheduling` | `ScheduledTask`, `interval()`, `cron()`, `cron_schedule()` |
| `events` | `EventBus` — typed in-process pub/sub |
| `realtime` | WebSocket + SSE support (`WsConnections`, `WebSocketHandler`, `sse_channel()`) |
| `application-services` | Bundled: `cache`, `scheduling`, `events`, `realtime` |

## Distributed Systems
| Flag | Enables |
|------|---------|
| `queues` | Message queue abstractions |
| `microservices` | Microservice patterns |
| `cqrs` | Command Query Responsibility Segregation |
| `sagas` | Saga orchestration patterns |
| `grpc` | gRPC server/client (via `tonic`) |
| `graphql` | GraphQL server (via `async-graphql`) |
| `distributed` | Bundled: `queues`, `microservices`, `cqrs`, `sagas`, `grpc`, `graphql` |

## Transport
| Flag | Enables |
|------|---------|
| `transport-redis` | Redis message transport |
| `transport-rabbitmq` | RabbitMQ message transport (via `lapin`) |
| `transport-kafka` | Kafka message transport |

## Ecosystem
| Flag | Enables |
|------|---------|
| `plugins` | Plugin system infrastructure |
| `devtools` | Development tools (JSON helpers, diagnostics) |
| `plugin-ecosystem` | Bundled: `plugins`, `devtools` |

## Observability
| Flag | Enables |
|------|---------|
| `metrics` | Prometheus metrics (`Counter`, `Gauge`, `Histogram`, `MetricsRegistry`) |
| `telemetry` | OpenTelemetry distributed tracing (OTLP export) |
| `logging` | Structured JSON logging (`TimeSeriesLayer`, `FileLogStorage`, `log` module) |

## Security
| Flag | Enables |
|------|---------|
| `security` | Bundled: `security-cors`, `security-rate-limit`, `security-headers`, `security-csrf` |
| `security-cors` | CORS middleware (`CorsMiddleware`) |
| `security-rate-limit` | Rate limiting (`RateLimitMiddleware`) |
| `security-headers` | Security headers (`SecurityHeadersMiddleware`) |
| `security-csrf` | CSRF protection |

## HTTP & API
| Flag | Enables |
|------|---------|
| `compression` | Gzip/brotli/zstd response compression (via `tower-http`) |
| `static-files` | Static file serving (via `tower-http`) |
| `multipart` | Multipart file uploads (`MultipartForm<T>`, `UploadedFile`) |
| `openapi` | OpenAPI schema generation (`#[api]`, `#[resp]`, `#[body(json = Type)]`) |

## Resilience
| Flag | Enables |
|------|---------|
| `resilience` | `RetryLayer`, `CircuitBreakerLayer` (Tower layers) |
| `resilience-ext` | `ConcurrencyLimitLayer` backpressure + bulkhead |

## Other
| Flag | Enables |
|------|---------|
| `hot-reload` | `ConfigurationLoader::watch()`, `ConfigWatcher<T>`, `FeatureToggle` |
| `backtrace` | `std::backtrace::Backtrace` capture on `HttpError` |
| `uuid` | `ParseUUIDPipe`, `parse_uuid()` for UUID path parameters |
| `validation` | `ValidationPipe`, request body validation |
| `versioning` | `VersionMetadata`, `VersioningStrategy` for API versioning |
| `serialization` | `Serializable`, `SerializeInterceptor`, field-level role rules |
| `cron` | Cron expression support in scheduling (`cron()`, `cron_schedule()`) |
| `custom-decorators` | `create_param_decorator!` macro (note: custom decorators work without this flag) |

## Production Features

These are always available (no feature flag required):

| Feature | API |
|---------|-----|
| Error envelope | `FrameworkResponse::error_with_tracing()` — includes `request_id`, `timestamp_ms` |
| Paginated response | `FrameworkResponse::paginated()` — `{"items","total","offset","limit"}` |
| Per-route timeout | `RouteDefinition::timeout(duration)` — overrides global adapter timeout |
| Feature gate guard | `FeatureGateGuard::new("feature-name")` — gates routes behind runtime toggles |
| Cache prefix invalidation | `Cache::remove_by_prefix(prefix)` — invalidates all keys starting with prefix |
| Rate limit key resolver | `RateLimitMiddleware::key_resolver(...)` — custom rate limit keys |
| TCP connection limit | `AxumAdapter::max_connections(n)` — prevents socket exhaustion |
| Task pause/resume | `ScheduledTask::pause()` / `resume()` — runtime task control |
| Error counter metric | `ironic_http_errors_total` — auto-incremented on 5xx in Prometheus scrape |
| Content negotiation | `RequestContext::accepts_json()` / `preferred_content_type()` |
| BeforeShutdown fix | Runs BEFORE server stops accepting connections |
| Provider health | `Container::health()` → `ProviderHealthSummary` |
| Dead-letter queue | `EventBus::drain_dead_letters()` — captures undelivered events |
| Per-endpoint status metrics | `ironic_http_endpoint_status_total{status="2xx/4xx/5xx"}` |
| Hot-reload config injection | `Reloadable<T>` — watch channel for runtime config updates |
| Post-bootstrap overrides | `Container::with_override(provider)` — hot-swap providers |
| Streaming body | `FrameworkBody::Stream(Arc<Vec<u8>>)` + `FrameworkResponse::from_stream()` |
| Dynamic module hooks | `OnModuleLoad` / `OnModuleUnload` — runtime module lifecycle |
