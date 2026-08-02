---
title: Releases
description: Version history and release notes for the Ironic framework.
---

# Releases

## Current version: v1.2.8

All notable changes to Ironic are documented here. The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

| Version | Date | Highlights |
|---------|------|-----------|
| [v1.2.8](/docs/releases/v1.2.x) | 2026-08-01 | add fetch_latest and poll_interval_ms to KafkaClientConfig and TransportConfig (6a91e13) |
| [v1.2.7](/docs/releases/v1.2.x) | 2026-07-28 | Auto-validation pipe ;  macro now automatically validates , ,  via |
| [v1.2.6](/docs/releases/v1.2.x) | 2026-07-28 | #[event_handler] auto_register is now default — use manual_register to opt out |
| [v1.2.5](/docs/releases/v1.2.x) | 2026-07-28 | clippy warnings (borrowed_box, collapsible_if, map_unwrap_or, used_underscore_binding) (ec16ebc) |
| [v1.2.4](/docs/releases/v1.2.x) | 2026-07-28 | EventClient injection in #[event_handler(transport, auto_register)] — handlers can receive Arc<EventClient> as second param to emit events |
| [v1.2.3](/docs/releases/v1.2.x) | 2026-07-28 | Transport-agnostic EventClient/EventServer DI providers with auto-connect/listen lifecycle, TransportConfig/TransportKind config, and transport auto_register support in #[event_handler] |
| [v1.2.2](/docs/releases/v1.2.x) | 2026-07-27 | add support for GraphQL and gRPC resource generation in resource module (0a2300e) |
| [v1.2.1](/docs/releases/v1.2.x) | 2026-07-27 | add serde dependency for serialization in GraphQL, gRPC, HTTP, and monorepo configurations (3a627d3) |
| [v1.2.0](/docs/releases/v1.2.x) | 2026-07-27 | add serde dependency for serialization and re-export in the library (7bc0ff9) |
| [v1.1.9](/docs/releases/v1.1.x) | 2026-07-27 | update dependencies and enhance GraphQL integration in project generator (579a213) |
| [v1.1.8](/docs/releases/v1.1.x) | 2026-07-26 | ironic generate app --grpc for gRPC microservice scaffold |
| [v1.1.7](/docs/releases/v1.1.x) | 2026-07-26 | enhance app generation with platform module and logging configuration (d46a656) |
| [v1.1.6](/docs/releases/v1.1.x) | 2026-07-26 | Simplify generated app template to minimal skeleton |
| [v1.1.5](/docs/releases/v1.1.x) | 2026-07-25 | generate_app creates revallens-style microservice with platform/welcome/example module, per-app Dockerfile/.env, unique port |
| [v1.1.4](/docs/releases/v1.1.x) | 2026-07-25 | New app scaffolding matches RivalLens structure: multi-line #[module()], repositories/tests in module template |
| [v1.1.3](/docs/releases/v1.1.x) | 2026-07-25 | enhance app generation with health module and controller scaffolding (aae2a7b) |
| [v1.1.2](/docs/releases/v1.1.x) | 2026-07-25 | add package argument to CargoArgs for monorepo support and update documentation (3aaed0b) |
| [v1.1.1](/docs/releases/v1.1.x) | 2026-07-25 | implement MQTT and NATS live transports, Federation helper, MS pipeline docs (80808bb) |
| [v1.1.0](/docs/releases/v1.1.x) | 2026-07-23 | release.sh: prefer [Unreleased] content over git log when non-empty |
| [v1.0.9](/docs/releases/v1.0.x) | 2026-07-21 | add documentation for backtrace and UUID features, and implement message queues and saga orchestration (53406ae) |
| [v1.0.8](/docs/releases/v1.0.x) | 2026-07-18 | add pagination extractor and SQL error mapping utilities (7b4fcdc) |
| [v1.0.7](/docs/releases/v1.0.x) | 2026-07-18 | add pagination extractor and SQL error mapping utilities (7b4fcdc) |
| [v1.0.6](/docs/releases/v1.0.x) | 2026-07-18 | implement blog API example with CRUD operations and JWT authentication (a2f68ca) |
| [v1.0.5](/docs/releases/v1.0.x) | 2026-07-18 | implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf) |
| [v1.0.4](/docs/releases/v1.0.x) | 2026-07-17 | add async test macro to simplify testing without external dependencies (5b03b7f) |
| [v1.0.3](/docs/releases/v1.0.x) | 2026-07-17 | add async test macro to simplify testing without external dependencies (5b03b7f) |
| [v1.0.2](/docs/releases/v1.0.x) | 2026-07-17 | enhance release workflow with version detection and conditional execution (f79b4db) |
| [v1.0.1](/docs/releases/v1.0.x) | 2026-07-17 | single version source of truth in docs/lib/constants.ts (0f01d78) |
| [v1.0.0](/docs/releases/v1.0.x) | 2026-07-17 | GitHub Actions CI with matrix testing across stable and nightly Rust (e3e863c) |
| [v0.5.0](/docs/releases/v0.5.x) | 2026-07-16 | update alias for Decorator command from 'd' to 'de' (e3e863c) |
| [v0.4.9](/docs/releases/v0.4.x) | 2026-07-16 | implement CI/CD pipeline, security auditing, and operational endpoints (e5537f2) |
| [v0.4.8](/docs/releases/v0.4.x) | 2026-07-16 | add database migration commands and update documentation (1e3db79) |
| [v0.4.7](/docs/releases/v0.4.x) | 2026-07-16 | enhance release script and project generator for better version handling and documentation sync (a8e859e) |
| [v0.4.6](/docs/releases/v0.4.x) | 2026-07-16 | update version to 0.4.6 and enhance OpenAPI support with new attributes (f088ce6) |
| [v0.4.5](/docs/releases/v0.4.x) | 2026-07-16 | `openapi` feature flag — OpenAPI/Swagger module is now feature-gated (was always compiled) and included in default features |
| [v0.4.4](/docs/releases/v0.4.x) | 2026-07-16 | enhance update command to automatically upgrade to the latest version (24228b6) |
| [v0.4.3](/docs/releases/v0.4.x) | 2026-07-16 | update default server host to 0.0.0.0 in multiple examples (435807c) |
| [v0.4.2](/docs/releases/v0.4.x) | 2026-07-16 | enable hot-reload feature in Cargo.toml (a87a424) |
| [v0.4.1](/docs/releases/v0.4.x) | 2026-07-15 | add repository generation support in CLI and refactor todo app (09f74f4) |
| [v0.4.0](/docs/releases/v0.4.x) | 2026-07-15 | Implement production readiness improvements for Ironic (2bf4555) |
| [v0.3.9](/docs/releases/v0.3.x) | 2026-07-15 | add release notes for v0.3.9 and enhance release script documentation (08592c9) |
| [v0.3.8](/docs/releases/v0.3.x) | 2026-07-15 | enhance observability section with health checks, metrics, and tracing documentation (cf2cc42) |
| [v0.3.7](/docs/releases/v0.3.x) | 2026-07-15 | add global middleware support for application builder and enhance security features (7113eef) |
| [v0.3.6](/docs/releases/v0.3.x) | 2026-07-15 | update validation pipes documentation with comprehensive examples and improved descriptions (c56dc5b) |
| [v0.3.5](/docs/releases/v0.3.x) | 2026-07-15 | refactor authentication test file structure and update module imports (97720ac) |
| [v0.3.4](/docs/releases/v0.3.x) | 2026-07-15 | remove unused integration module from tests (61aa525) |
| [v0.3.3](/docs/releases/v0.3.x) | 2026-07-15 | auto-add required dependencies to Cargo.toml during module registration (e8de7ce) |
| [v0.3.2](/docs/releases/v0.3.x) | 2026-07-15 | update documentation link in navigation component for clarity (d9eafaf) |
| [v0.3.1](/docs/releases/v0.3.x) | 2026-07-15 | allow needless raw string hashes and restore GenerationReport import in ready_resource.rs (583ba86) |
| [v0.3.0](/docs/releases/v0.3.x) | 2026-07-15 | Initial release |

Full changelog: [CHANGELOG.md](https://github.com/ironic-org/ironic/blob/main/CHANGELOG.md)

## Versioning policy

Since v1.0.0, Ironic follows strict [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **Major** (1 → 2): Breaking API changes, significant design shifts
- **Minor** (1.0 → 1.1): New features, non-breaking additions
- **Patch** (1.0.0 → 1.0.1): Bug fixes, docs, small improvements

### What requires a major bump

- Removal or rename of public APIs
- Changes to trait bounds on public traits
- Changes to default feature sets
- MSRV (Minimum Supported Rust Version) bumps
- Upgrade of a re-exported dependency major version

### What is NOT breaking

- Adding new APIs, modules, or features
- Deprecating existing APIs (with warning)
- Internal refactors

Previous releases: [v0.5.x](./v0.5.x) | [v0.4.x](./v0.4.x) | [v0.3.x](./v0.3.x)
