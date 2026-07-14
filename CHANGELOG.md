# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
