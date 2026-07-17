---
title: Releases
description: Version history and release notes for the Ironic framework.
---

# Releases

## Current version: v1.0.1

All notable changes to Ironic are documented here. The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

| Version | Date | Highlights |
|---------|------|-----------|
| [v1.0.1](/blog/v1.0.1) | 2026-07-17 | single version source of truth in docs/lib/constants.ts |
| [v1.0.0](/blog/v1.0.0) | 2026-07-17 | Ironic 1.0.0 is here — production-ready with comprehensive CI/CD pipelines, matrix testing across stable and nightly Rust, automated crates.io publishing, production release guide, blog-api cross-module DI example, and 200+ passing tests. |
| [v0.5.0](/blog/v0.5.0) | 2026-07-16 | update alias for Decorator command from 'd' to 'de' |
| [v0.4.9](/blog/v0.4.9) | 2026-07-16 | implement CI/CD pipeline, security auditing, and operational endpoints |
| [v0.4.8](/blog/v0.4.8) | 2026-07-16 | add database migration commands and update documentation |
| [v0.4.7](/blog/v0.4.7) | 2026-07-16 | enhance release script and project generator for better version handling and documentation sync |
| [v0.4.6](/blog/v0.4.6) | 2026-07-16 | Release v0.4.6 |
| [v0.4.5](/blog/v0.4.5) | 2026-07-16 | Release v0.4.5 |
| [v0.4.4](/blog/v0.4.4) | 2026-07-16 | Release v0.4.4 |
| [v0.4.3](/blog/v0.4.3) | 2026-07-16 | Release v0.4.3 |
| [v0.4.2](/blog/v0.4.2) | 2026-07-16 | Release v0.4.2 |
| [v0.4.1](/blog/v0.4.1) | 2026-07-15 | Release v0.4.1 |
| [v0.4.0](/blog/v0.4.0) | 2026-07-15 | Multipart uploads, Redis sessions, OAuth2 callback handler, backpressure, config hot-reload, error backtraces, and 15+ documentation pages |
| [v0.3.9](/blog/v0.3.9) | 2026-07-15 | Release script now auto-generates blog posts, updates BlogIndex, and syncs releases pages on every release. |
| [v0.3.8](/blog/v0.3.8) | 2026-07-15 | Production-ready defaults ship with every new project. Security headers, rate limiting, CORS, and a fixed Dockerfile out of the box. |
| [v0.3.7](/blog/v0.3.7) | 2026-07-15 | The FrameworkApplicationBuilder now supports .middleware() for registering global middleware from main.rs. |
| [v0.3.6](/blog/v0.3.6) | 2026-07-15 | Deeper validation docs, new auth/basic CRUD example apps, and expanded project scaffolding. |
| [v0.3.5](/blog/v0.3.5) | 2026-07-15 | Refactored auth test file structure for better module organization. |
| [v0.3.4](/blog/v0.3.4) | 2026-07-15 | Documentation site deployed with SPA fallback, integration test paths fixed. |
| [v0.3.3](/blog/v0.3.3) | 2026-07-15 | The CLI now auto-adds required dependencies when generating modules. |
| [v0.3.0](/blog/v0.3.0) | 2026-07-15 | The first stable releases of Ironic: module system, DI container, controller routing, CLI generator, and testing harness. |

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
