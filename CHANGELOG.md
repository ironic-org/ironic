# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Changed

- Renamed project from "RustFrame" to "Ironic"
- Internal `rustframe_*` crate aliases renamed to `ironic_*`
- MSRV bumped from 1.85 to 1.97
- Dependency updates: diesel 2.2.12→2.3.11, jsonwebtoken→9 (pinned), time 0.3.45→0.3.47, hickory-proto 0.25.2→0.26.1
- Fixed 6 Rust 1.97 clippy warnings

### Security

- `.cargo/audit.toml` added to ignore unfixable RUSTSEC-2023-0071 (rsa, transitive via oauth2)
- CI supply-chain job runs `cargo audit` and `cargo deny check`
