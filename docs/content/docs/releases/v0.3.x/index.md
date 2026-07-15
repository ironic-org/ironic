---
title: v0.3.x
description: Complete changelog and release notes for the Ironic v0.3.x stable series.
---

# v0.3.x — Current Stable Series

All versions in the v0.3.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v0.3.9 — July 15, 2026

### Added
- Blog section with release notes, deep-dive articles, and version changelog
- Docs comparison table (NestJS, Axum, Actix Web, Loco, Ironic)
- Release releases page with full version history
- Navigation and footer on blog pages
- GitHub stars and forks badge in nav header

### Fixed
- Release script path: `getting-started.md` → `getting-started/getting-started.md`
- Blog links use React Router `Link` for proper base path on GitHub Pages
- SPA 404 fallback for blog direct URL access

### Changed
- Release script now auto-creates blog post, updates BlogIndex and releases on each version
- Docs CI skips full Rust build for docs-only changes
- GitHub Discussions links → Issues (Discussions not yet enabled)

---

## v0.4.0 — July 15, 2026

### Added
- Update release script to bump current version and add new release to index (481aa56)\n- Update release script to publish to crates.io before git push (7e1ad7a)\n- Update changelog and add blog post for v0.4.0 release (dbd6837)\n- Update telemetry and tracing documentation (fc34232)\n- Implement production readiness improvements for Ironic (2bf4555)\n- Add ready-resource generator for production-grade authentication module (ea28f4c)\n- Add production readiness improvements across multiple components (948341b)\n- add blog post on lifecycle hooks in axum integration (805a566)\n- add blog posts on OnceCell-based singletons, sagas, scope violations, static plugin system, and two-phase route compilation (de3126e)\n- refactor blog and releases index update logic in release script (8102c9a)\n- update release notes and automate blog post generation for v0.3.9 (cb654ba)\n- update changelog and release notes for v0.3.9 (699a8d6)\n- add release notes for v0.3.9 and enhance release script documentation (08592c9)\n- enhance release script to create blog post and update releases documentation (66b0a0a)\n
### Fixed
- Correct Prometheus output format code block in metrics documentation (1475744)\n- update background styles in BlogIndex and BlogPage components (82f3c58)\n
### Changed
- Add new blog posts on various Ironic features and improvements (04a9ae9)\n- Add blog posts on handler dispatch, injectable generation, and feature flags (fb37128)\n

---


## v0.4.1 — July 15, 2026

### Added
- Enhance production readiness with telemetry, metrics, health checks, and more (3feff61)\n- add repository generation support in CLI and refactor todo app (09f74f4)\n- Add comprehensive documentation for Todo API, database migrations, schema, architecture, deployment, and development setup (5034e24)\n- initialize todo application with Ironic framework (4b19726)\n- Enhance database integration documentation with setup instructions and examples (afea150)\n- Add S3 upload documentation and update meta.json to include new page (630047e)\n- Add configuration and migrations metadata, update advanced pages (16d2473)\n- Update blog post for v0.4.0 with production readiness and enterprise features (b5790de)\n- Update release notes for v0.4.0 with detailed features and improvements (336c954)\n- Refactor imports in error and lib modules for better organization (199bc4f)\n
### Fixed
- update path for ironic dependency in todo app example (18d76f0)\n- Update configuration file names in tests for consistency (cc98918)\n- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)\n- Update actions/checkout version to v5 in CI workflow (e4c9e5d)\n- Clean stale cache artifacts in CI workflow (56a9b2c)\n- Remove redundant import and reorganize imports for clarity (1a4349d)\n
### Changed
- remove unused dependencies and example from Cargo configuration (5a2dcf0)\n- release v0.4.1 with new features, improvements, and documentation updates (4784cdb)\n- streamline code structure and improve readability across multiple files (3b7b0a2)\n

---


## v0.4.2 — July 16, 2026

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)\n- remove redundant command for cleaning stale test cache artifacts (e560244)\n- update release script to check if version is published on crates.io before proceeding (d188dfc)\n
### Changed
- enhance getting started guide with project structure details (eb6ebeb)\n

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
| v0.1.x | Deprecated | First public release with full architecture foundation |

Read the full pre-release story: [The pre-release journey](/blog/v0.1.x-v0.2.x)
