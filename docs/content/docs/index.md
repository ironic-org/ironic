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
| Validation pipes | `validation` | [`ValidationPipe`](https://docs.rs/ironic) |
| Exception filters | _(always on)_ | [`ExceptionFilter` trait](https://docs.rs/ironic) |
| API versioning | `versioning` | [`VersionMetadata`](https://docs.rs/ironic) |
| Response serialization | `serialization` | [`#[derive(Serializable)]`](https://docs.rs/ironic) |
| CORS | `security-cors` | [Security](/docs/security#cors) |
| Security headers | `security-headers` | [Security](/docs/security#security-headers) |
| Rate limiting | `security-rate-limit` | [Security](/docs/security#rate-limiting) |
| CSRF | `security-csrf` | [Security](/docs/security#csrf) |
| Compression | `compression` | [Security](/docs/security#response-compression) |

The framework remains usable through explicit Rust APIs; procedural macros only generate calls to
those public contracts.
