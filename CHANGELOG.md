# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

## [v0.2.9] - 2026-07-15

### Added
- add file upload and email modules with respective generators (3bc21f8)\n- add comprehensive authentication module with various strategies (8dc08b2)\n- add ready-resource generator for complete authentication module with variants (81e9e9f)\n
### Fixed
- update error code reference in rate limit middleware (603fcae)\n- update permissions and restructure GitHub Actions workflow for documentation deployment (f63caf3)\n- add permissions section for GitHub Actions workflow to enable content writing (0800ae6)\n- adjust formatting of router creation in main.tsx for improved readability (e76ab60)\n- simplify GitHub Actions workflow for deploying documentation to GitHub Pages (5841216)\n- restructure GitHub Actions workflow for deploying documentation to GitHub Pages (1856566)\n- update link in HeroSection to point to the getting started page (0890f33)\n

## [v0.2.9] - 2026-07-14

### Fixed
- ensure stale local tags are deleted before creating new ones and improve push error handling (da47b2a)\n

## [v0.2.8] - 2026-07-14

### Added
- update dotenv example with placeholder values and improve CSRF cookie/header name validation (da96fc8)\n
### Fixed
- handle poisoned mutex locks in metrics, resilience, security modules (399821a)\n
### Changed
- streamline CorsConfig initialization in tests (9517f27)\n- update CORS configuration tests to reflect default deny behavior and explicit origin allowance (90e16ad)\n

## [v0.2.8] - 2026-07-14

### Added
- add workspace command to print project information (f4edadd)\n

## [v0.2.8] - 2026-07-14

### Added
- add workspace command to print project information (f4edadd)\n

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
