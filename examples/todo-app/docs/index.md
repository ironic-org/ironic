# todo-example

A production-ready Todo API built with [Ironic](https://github.com/ironic-org/ironic) v0.4.0.

## Quick links

- [Setup guide](guide/setup.md)
- [Architecture overview](guide/architecture.md)
- [API reference](api/todos.md)
- [Database schema](database/schema.md)
- [Deployment](guide/deployment.md)

## Features

| Layer | Implementation |
|---|---|
| HTTP framework | Ironic (Axum adapter) |
| Database | PostgreSQL via SQLx with migrations |
| Validation | garde on DTOs |
| Logging | tracing + tracing-subscriber (structured, env-filtered) |
| Metrics | Prometheus via MetricsLayer |
| Security | CORS, rate limiting, security headers |
| Compression | Automatic gzip/brotli via AxumAdapter |
| Caching | In-memory cache interceptor |
| Scheduling | Background task support |

## Stack

```
┌─────────────────────────────────────────┐
│              todo-example                │
├─────────────────────────────────────────┤
│  Ironic Framework (Axum adapter)        │
├─────────────────────────────────────────┤
│  SQLx ORM → PostgreSQL                  │
└─────────────────────────────────────────┘
```
