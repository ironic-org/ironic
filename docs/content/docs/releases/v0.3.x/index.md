---
title: v0.3.x
description: Documentation for Ironic v0.3.x — the current stable release series.
---

# v0.3.x Documentation

This is the current stable version of Ironic. All docs at [ironic.dev/docs](https://ironic.dev/docs) reflect this version.

## What's in v0.3.x

- Module system with validated graphs and compile-time checks
- Dependency injection (singleton, transient, request-scoped)
- Controller routing with `#[get]`/`#[post]`/`#[put]`/`#[delete]`
- Full request pipeline: Middleware → Guards → Interceptors → Pipes → Handler
- Socket-free integration testing with `TestApplication`
- CLI scaffolding with `ironic new` and `ironic generate`
- Security middleware: CORS, rate limiting, CSRF, security headers
- Lifecycle hooks with graceful shutdown
- Axum platform adapter with compression, body limit, timeout
- OpenAPI 3.1 route discovery and schema generation

## Previous versions

| Version | Status |
|---------|--------|
| v0.3.x | **Current stable** |
| v0.2.x | Deprecated |
| v0.1.0 | Preview release |
