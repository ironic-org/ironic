---
title: Ironic
description: A modular, type-safe Rust application framework built on Axum.
---

# Ironic documentation

Ironic provides explicit modules, dependency injection, controllers, request pipelines,
lifecycle hooks, testing utilities, and an Axum adapter without runtime reflection or global
mutable state.

## Start here

- [Getting started](/docs/getting-started/getting-started) — build and run the first application.
- [CLI reference](/docs/getting-started/cli) — create, run, test, and generate application source.
- [Fundamentals](/docs/core/fundamentals) — modules, providers, controllers, and lifecycle.
- [Configuration](/docs/core/configuration) — typed sources, validation, and redacted secrets.
- [Security](/docs/http-api/security) — CORS, rate limiting, security headers, CSRF, compression, and secrets.
- [Examples](/docs/more/examples) — REST, validation, error handling, versioning, serialization, and testing.
- [Benchmarks](/docs/more/benchmarks) — reproducible framework overhead measurements.

## Feature overview

| Area | Feature flags | Documentation |
|------|---------------|---------------|
| Validation pipes | `validation` | [Validation pipes](/docs/http-api/validation-pipes) |
| Exception filters | _(always on)_ | [Exception filters](/docs/http-api/exception-filters) |
| API versioning | `versioning` | [API versioning](/docs/http-api/api-versioning) |
| Response serialization | `serialization` | [Response serialization](/docs/http-api/response-serialization) |
| Compression | `compression` | [Compression](/docs/http-api/compression) |
| CORS | `security-cors` | [Security](/docs/http-api/security#cors) |
| Security headers | `security-headers` | [Security](/docs/http-api/security#security-headers) |
| Rate limiting | `security-rate-limit` | [Security](/docs/http-api/security#rate-limiting) |
| CSRF | `security-csrf` | [Security](/docs/http-api/security#csrf) |
| Cache decorators | `cache` | [Cache decorators](/docs/performance/cache-decorators) |
| Task scheduling | `scheduling` | [Task scheduling](/docs/performance/scheduling) |
| Cron scheduling | `cron` | [Task scheduling](/docs/performance/scheduling#cron-expression-tasks) |
| WebSocket gateways | `realtime` | [WebSocket gateways](/docs/advanced/websocket-gateways) |
| Custom decorators | `custom-decorators` | [Custom decorators](/docs/modules/custom-decorators) |
| Dynamic modules | _(always on)_ | [Dynamic modules](/docs/modules/dynamic-modules) |
| Optional DI deps | _(always on)_ | [Dependency management](/docs/core/dependency-management#optional-di-dependencies) |
| Transport adapters | `transport-*` | [Distributed](/docs/performance/distributed#microservice-transports) |
| Database integrations | `database` | [Database integrations](/docs/data-auth/database-integrations) |
| Authentication | `authentication` | [Authentication](/docs/data-auth/authentication) |
| OpenAPI | _(always on)_ | [OpenAPI](/docs/http-api/openapi) |

The framework remains usable through explicit Rust APIs; procedural macros only generate calls to
those public contracts.
