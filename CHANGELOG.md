# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.4.8] - 2026-07-16

### Added
- add database migration commands and update documentation (1e3db79)\n
### Fixed
- improve formatting and readability in migration and project generation code (37a696c)\n- enhance API documentation for authentication endpoints (acdf3d1)\n- enhance OpenAPI attributes and improve controller documentation (e27518d)\n
### Changed
- Add robots.txt and site.webmanifest for SEO and PWA support (d21bb8f)\n- Implement code changes to enhance functionality and improve performance (57a33f2)\n

## [v0.4.7] - 2026-07-16

### Fixed
- enhance release script and project generator for better version handling and documentation sync (a8e859e)\n

## [v0.4.6] - 2026-07-16

### Added
- update version to 0.4.6 and enhance OpenAPI support with new attributes (f088ce6)\n
### Fixed
- comment out database module by default with setup guide (a0612d4)\n

## [v0.4.5] - 2026-07-16

### Added

- `openapi` feature flag — OpenAPI/Swagger module is now feature-gated (was always compiled) and included in default features
- `logging` feature — structured time-series logging with `FileLogStorage` (`.logs/YYYY-MM-DD.jsonl`), `LogStorage` trait for pluggable backends, `TimeSeriesLayer` capturing all `tracing` events, and `ironic::log::{info, warn, error, debug, trace}` re-exports
- `logging` feature included in generated project template

### Fixed

- Generated project template now calls `.configure_router()` before `.with_openapi()` (method exists on `AxumAdapter`, not `OpenApiAxumAdapter`)
- Generated project now includes `sqlx` and `tracing` as direct dependencies for the database module
- `extern crate` aliases annotated with `#[allow(unused_extern_crates)]` to fix builds without default features
- Various code formatting fixes

## [v0.4.4] - 2026-07-16

### Added
- enhance update command to automatically upgrade to the latest version (24228b6)\n

## [v0.4.3] - 2026-07-16

### Fixed
- update default server host to 0.0.0.0 in multiple examples (435807c)\n- update latest version in BlogIndex to v0.4.2 (2ca67ef)\n

## [v0.4.2] - 2026-07-16

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)\n- remove redundant command for cleaning stale test cache artifacts (e560244)\n- update release script to check if version is published on crates.io before proceeding (d188dfc)\n
### Changed
- enhance getting started guide with project structure details (eb6ebeb)\n

## [v0.4.1] - 2026-07-15

### Added
- add repository generation support in CLI and refactor todo app (09f74f4)\n- Add comprehensive documentation for Todo API, database migrations, schema, architecture, deployment, and development setup (5034e24)\n- initialize todo application with Ironic framework (4b19726)\n- Enhance database integration documentation with setup instructions and examples (afea150)\n- Add S3 upload documentation and update meta.json to include new page (630047e)\n- Add configuration and migrations metadata, update advanced pages (16d2473)\n- Update blog post for v0.4.0 with production readiness and enterprise features (b5790de)\n- Update release notes for v0.4.0 with detailed features and improvements (336c954)\n- Refactor imports in error and lib modules for better organization (199bc4f)\n
### Fixed
- Update configuration file names in tests for consistency (cc98918)\n- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)\n- Update actions/checkout version to v5 in CI workflow (e4c9e5d)\n- Clean stale cache artifacts in CI workflow (56a9b2c)\n- Remove redundant import and reorganize imports for clarity (1a4349d)\n
### Changed
- streamline code structure and improve readability across multiple files (3b7b0a2)\n

## [v0.4.0] - 2026-07-15

### Added
- Implement production readiness improvements for Ironic (2bf4555)\n- Add ready-resource generator for production-grade authentication module (ea28f4c)\n- Add production readiness improvements across multiple components (948341b)\n- add blog post on lifecycle hooks in axum integration (805a566)\n- add blog posts on OnceCell-based singletons, sagas, scope violations, static plugin system, and two-phase route compilation (de3126e)\n- refactor blog and releases index update logic in release script (8102c9a)\n- update release notes and automate blog post generation for v0.3.9 (cb654ba)\n- update changelog and release notes for v0.3.9 (699a8d6)\n- add release notes for v0.3.9 and enhance release script documentation (08592c9)\n- enhance release script to create blog post and update releases documentation (66b0a0a)\n
### Fixed
- update background styles in BlogIndex and BlogPage components (82f3c58)\n
### Changed
- Add new blog posts on various Ironic features and improvements (04a9ae9)\n- Add blog posts on handler dispatch, injectable generation, and feature flags (fb37128)\n

## [v0.3.9] - 2026-07-15

### Added
- add release notes for v0.3.9 and enhance release script documentation (08592c9)\n- enhance release script to create blog post and update releases documentation (66b0a0a)\n

## [v0.3.8] - 2026-07-15

### Added
- enhance observability section with health checks, metrics, and tracing documentation (cf2cc42)\n- update server host in dotenv example and Dockerfile for better accessibility (381f0eb)\n- update Dockerfile generation to use kebab-case project name (137202a)\n

## [v0.3.7] - 2026-07-15

### Added
- add global middleware support for application builder and enhance security features (7113eef)\n

## [v0.3.6] - 2026-07-15

### Added
- update validation pipes documentation with comprehensive examples and improved descriptions (c56dc5b)\n- add basic and auth API examples with CRUD functionality (b10e11e)\n- enhance project manifest with additional dependencies and security features (613d478)\n
### Fixed
- allow dead code warnings for unit tests in authentication module (77c5c02)\n
### Changed
- update version to 0.3.6 and remove unused API examples from workspace (914a74d)\n

## [v0.3.5] - 2026-07-15

### Fixed
- refactor authentication test file structure and update module imports (97720ac)\n

## [v0.3.4] - 2026-07-15

### Fixed
- remove unused integration module from tests (61aa525)\n- update integration test file paths for auth modules (db79152)\n- docs pages deployment with .nojekyll and SPA fallback (310efb2)\n

## [v0.3.3] - 2026-07-15

### Added
- auto-add required dependencies to Cargo.toml during module registration (e8de7ce)\n
### Fixed
- format manual instructions for clarity in module registration (4f55008)\n

## [v0.3.2] - 2026-07-15

### Fixed
- update documentation link in navigation component for clarity (d9eafaf)\n- update parameter names for consistency in auth module decorators and guards (18009e6)\n

## [v0.3.1] - 2026-07-15

### Fixed
- allow needless raw string hashes and restore GenerationReport import in ready_resource.rs (583ba86)\n
### Changed
- bump version to 0.3.1 in Cargo.toml and Cargo.lock (d4d7b20)\n- reorder module imports for consistency in ready_resource.rs (7fd6159)\n- update module imports and improve code readability in ready_resource.rs (d7d944f)\n

## [v0.3.0] - 2026-07-15

- Initial release


## [v0.2.9] - 2026-07-15

### Added
- update changelog and add new ready-resource documentation for authentication, file upload, and email modules (07f6232)\n- add file upload and email modules with respective generators (3bc21f8)\n- add comprehensive authentication module with various strategies (8dc08b2)\n- add ready-resource generator for complete authentication module with variants (81e9e9f)\n
### Fixed
- update error code reference in rate limit middleware (603fcae)\n- update permissions and restructure GitHub Actions workflow for documentation deployment (f63caf3)\n- add permissions section for GitHub Actions workflow to enable content writing (0800ae6)\n- adjust formatting of router creation in main.tsx for improved readability (e76ab60)\n- simplify GitHub Actions workflow for deploying documentation to GitHub Pages (5841216)\n- restructure GitHub Actions workflow for deploying documentation to GitHub Pages (1856566)\n- update link in HeroSection to point to the getting started page (0890f33)\n
### Changed
- simplify register_module function signature (5733b4f)\n

## [v0.2.8] - 2026-07-14

### Added
- update dotenv example with placeholder values and improve CSRF cookie/header name validation (da96fc8)\n
### Fixed
- handle poisoned mutex locks in metrics, resilience, security modules (399821a)\n
### Changed
- streamline CorsConfig initialization in tests (9517f27)\n- update CORS configuration tests to reflect default deny behavior and explicit origin allowance (90e16ad)\n

## [v0.2.7] - 2026-07-14

### Added
- add dotenvy support for configurable server host and port in main source (846e89b)\n

## [v0.2.6] - 2026-07-14

### Added
- improve changelog generation with formatted entries and enhanced parsing (fdcac78)\n- add changelog generation to release script (0653753)\n- enhance project scaffold generation with example module and CI workflow (13a29dc)\n
### Changed
- update version numbers to 0.2.5 in documentation and code (9408e57)\n

## [v0.2.5] - 2026-07-14nn### Addedn- feat: add changelog generation to release script (0653753)n- feat: enhance project scaffold generation with example module and CI workflow (13a29dc)nn### Changedn- chore: update version numbers to 0.2.5 in documentation and code (9408e57)n

## [v0.2.5] - 2026-07-14nn### Addedn- feat: add changelog generation to release script (0653753)n- feat: enhance project scaffold generation with example module and CI workflow (13a29dc)n

## [0.1.4] - 2026-07-13

### Added

- Initial open-source release
- Workspace with 9 crates + irony facade crate
- Module system (RFC 0001)
- Dependency injection (RFC 0002)
- Controller routing (RFC 0003)
- Request lifecycle pipeline (RFC 0004)
- Platform adapter boundary with Axum adapter (RFC 0005)
- CLI project scaffolding (`ironic new`)
- OpenAPI 3.1 route discovery and Swagger UI
- Health endpoints
- Request correlation spans
- Integration testing utilities
- Feature-gated database backends: SQLx, SeaORM, Diesel, MongoDB, Redis
- Feature-gated authentication: Argon2, JWT, OAuth2, sessions
- Feature-gated services: caching, scheduling, events, realtime, queues
- Feature-gated distributed features: microservices, CQRS, sagas, gRPC, GraphQL
- NestJS feature parity: security middleware (CORS, rate limiting, CSRF, security headers)
- NestJS feature parity: validation pipes (`ParseIntPipe`, `ParseFloatPipe`, `ParseBoolPipe`, `ValidationPipe`)
- NestJS feature parity: exception filters with route metadata access and scope precedence
- NestJS feature parity: API versioning (URI prefix, header, media type strategies)
- NestJS feature parity: response serialization with `#[exclude]` / `#[expose(role)]` field-level rules
- NestJS feature parity: compression middleware (gzip, brotli, deflate) via `tower-http`
- NestJS feature parity: WebSocket gateways with `#[web_socket_gateway]`, `#[subscribe_message]`, rooms, and broadcasting
- NestJS feature parity: microservice transport adapters for Redis, RabbitMQ, Kafka (feature-gated)
- NestJS feature parity: cache interceptor with `#[cache(ttl_secs = N)]` route attribute and `CacheMetadata`
- NestJS feature parity: cron scheduling with `cron_schedule()`, `#[cron]`, `#[interval]`, `#[timeout]` markers
- NestJS feature parity: global modules with `#[global]` attribute and `ModuleRef` runtime DI container access
- NestJS feature parity: optional dependencies via `#[injectable(optional = [Type, ...])]`
- NestJS feature parity: custom decorator support with `create_param_decorator!` macro
- New feature flags: `security`, `security-cors`, `security-rate-limit`, `security-headers`, `security-csrf`, `compression`, `versioning`, `serialization`, `validation`, `cron`, `custom-decorators`, `transport-redis`, `transport-rabbitmq`, `transport-kafka`

### Changed

- Renamed project from "RustFrame" to "Ironic"
- Internal `rustframe_*` crate aliases renamed to `ironic_*`
- MSRV bumped from 1.85 to 1.97
- Dependency updates: diesel 2.2.12→2.3.11, jsonwebtoken→9 (pinned), time 0.3.45→0.3.47, hickory-proto 0.25.2→0.26.1
- Fixed 6 Rust 1.97 clippy warnings

### Security

- `.cargo/audit.toml` added to ignore unfixable RUSTSEC-2023-0071 (rsa, transitive via oauth2)
- CI supply-chain job runs `cargo audit` and `cargo deny check`
