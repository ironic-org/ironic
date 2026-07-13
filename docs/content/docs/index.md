---
title: Ironic
description: A modular, type-safe Rust application framework built on Axum.
---

# Ironic documentation

Ironic provides explicit modules, dependency injection, controllers, request pipelines,
lifecycle hooks, testing utilities, and an Axum adapter without runtime reflection or global
mutable state.

## Start here

- [Getting started](/docs/getting-started) — build and run the first application.
- [CLI reference](/docs/cli) — create, run, test, and generate application source.
- [Fundamentals](/docs/fundamentals) — modules, providers, controllers, and lifecycle.
- [Configuration](/docs/configuration) — typed sources, validation, and redacted secrets.
- [Security](/docs/security) — CORS, rate limiting, security headers, CSRF, compression, and secrets.
- [Examples](/docs/examples) — REST, validation, error handling, versioning, serialization, and testing.
- [Benchmarks](/docs/benchmarks) — reproducible framework overhead measurements.

## Feature overview

| Area | Feature flags | Documentation |
|------|---------------|---------------|
| Validation pipes | `validation` | [Validation pipes](/docs/validation-pipes) |
| Exception filters | _(always on)_ | [Exception filters](/docs/exception-filters) |
| API versioning | `versioning` | [API versioning](/docs/api-versioning) |
| Response serialization | `serialization` | [Response serialization](/docs/response-serialization) |
| Compression | `compression` | [Compression](/docs/compression) |
| CORS | `security-cors` | [Security](/docs/security#cors) |
| Security headers | `security-headers` | [Security](/docs/security#security-headers) |
| Rate limiting | `security-rate-limit` | [Security](/docs/security#rate-limiting) |
| CSRF | `security-csrf` | [Security](/docs/security#csrf) |
| Cache decorators | `cache` | [Cache decorators](/docs/cache-decorators) |
| Task scheduling | `scheduling` | [Task scheduling](/docs/scheduling) |
| Cron scheduling | `cron` | [Task scheduling](/docs/scheduling#cron-expression-tasks) |
| WebSocket gateways | `realtime` | [WebSocket gateways](/docs/websocket-gateways) |
| Custom decorators | `custom-decorators` | [Custom decorators](/docs/custom-decorators) |
| Dynamic modules | _(always on)_ | [Dynamic modules](/docs/dynamic-modules) |
| Optional DI deps | _(always on)_ | [Dependency management](/docs/dependency-management#optional-di-dependencies) |
| Transport adapters | `transport-*` | [Distributed](/docs/distributed#microservice-transports) |
| Database integrations | `database` | [Database integrations](/docs/database-integrations) |
| Authentication | `authentication` | [Authentication](/docs/authentication) |
| OpenAPI | _(always on)_ | [OpenAPI](/docs/openapi) |

The framework remains usable through explicit Rust APIs; procedural macros only generate calls to
those public contracts.
