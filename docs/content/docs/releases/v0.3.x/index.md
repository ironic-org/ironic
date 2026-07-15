---
title: v0.3.x
description: Complete changelog and release notes for the Ironic v0.3.x stable series.
---

# v0.3.x — Current Stable Series

All versions in the v0.3.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v0.3.8 — July 15, 2026

### Added
- Global middleware stack in generated `main.rs` (SecurityHeaders, RateLimit, CORS, Metrics, Compression)
- `CORS_ORIGINS` JSON array parsing via `serde_json`
- `SERVER_HOST=0.0.0.0` default for Docker compatibility
- Enhanced observability docs with health checks, metrics, tracing

### Fixed
- Dockerfile binary name uses kebab-case project name (was hardcoded `./app`)
- Dockerfile `COPY` no longer overwrites the `/app/` directory

---

## v0.3.7 — July 15, 2026

### Added
- `FrameworkApplication::builder().middleware()` public API for global middleware
- `CompiledHttpApplication::extend_middleware()` for batch middleware registration
- `ironic::security::*` flat re-exports for cleaner imports

---

## v0.3.6 — July 15, 2026

### Added
- Comprehensive validation pipes documentation with `garde` rule reference
- Basic API example (CRUD controller + service + DTOs + tests)
- Auth API example (JWT, sessions, OAuth strategies)
- `serde`, `serde_json`, `garde`, `dotenvy` in generated `Cargo.toml`
- Optional features listed as comments in generated project manifest

### Fixed
- Dead code warnings in generated authentication test files

### Changed
- Default project features: `["security", "compression", "metrics", "validation"]`

---

## v0.3.5 — July 15, 2026

### Fixed
- Refactored authentication test file structure
- Updated module imports for auth test organization

---

## v0.3.4 — July 15, 2026

### Fixed
- Documentation site deployed with `.nojekyll` for GitHub Pages
- SPA fallback for client-side routing on docs
- Integration test paths updated for auth modules

---

## v0.3.3 — July 15, 2026

### Added
- CLI now auto-adds required Cargo dependencies when scaffolding modules
- `ironic generate` updates `Cargo.toml` automatically

### Fixed
- Improved manual instruction formatting during module generation

---

## v0.3.2 — July 15, 2026

### Fixed
- Documentation navigation links updated for clarity
- Auth module decorator and guard parameter consistency

---

## v0.3.1 — July 15, 2026

### Changed
- Module imports reordered for consistency in ready-resource generator
- Code readability improvements in generated resources

### Fixed
- Needless raw string hashes warning in generated code
- `GenerationReport` import restored in ready_resource.rs

---

## v0.3.0 — July 15, 2026

First stable release of the Ironic framework.

### Core
- Module graph compiler with validated providers, controllers, imports, and exports
- Dependency injection container with singleton and transient scopes
- `#[derive(Module)]` and `#[derive(Injectable)]` macros
- `OnModuleInit`, `OnApplicationBootstrap`, `OnModuleDestroy`, `OnApplicationShutdown`

### HTTP
- Controller routing via `#[controller]` + `#[get]/#[post]/#[put]/#[delete]`
- Request pipeline: Middleware → Guards → Interceptors → Extraction → Pipes → Handler
- Axum platform adapter with compression, body limit, and request timeout

### CLI
- `ironic new` — project scaffolding with full CRUD example
- `ironic generate` — module, controller, service, resource, middleware, guard, interceptor, pipe, decorator, filter, gateway, provider
- `ironic dev` — hot reload development server
- `ironic start`, `ironic build`, `ironic test`

### Testing
- `TestApplication` — socket-free, in-process HTTP testing
- `TestModule` — single-module isolation testing
- Provider overrides for mocking dependencies

### Benchmarks (arm64, release mode)
| Operation | Time |
|-----------|------|
| Module graph compilation | 866 ns/op |
| Route registration | 436 ns/op |
| Transient provider resolution | 157 ns/op |
| HTTP runtime startup | 555 ns/op |
| Ironic in-process request | 780 ns/op |

---

## Earlier versions

| Version | Status | Notes |
|---------|--------|-------|
| v0.2.x | Deprecated | Pre-release with ready-resource generators, auth modules, file upload, email |
| v0.1.0 | Preview | First public preview with core framework contracts |
