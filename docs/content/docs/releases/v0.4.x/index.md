---
title: v0.4.x
description: Complete changelog and release notes for the Ironic v0.4.x stable series.
---

## v0.4.9 — 2026-07-16

### Added
- implement CI/CD pipeline, security auditing, and operational endpoints (e5537f2)
- enhance observability with operational endpoints and health checks (0082bdb)

### Fixed
- improve documentation and formatting in build script and tests (5226611)

---
## v0.4.8 — 2026-07-16

### Added
- add database migration commands and update documentation (1e3db79)

### Fixed
- improve formatting and readability in migration and project generation code (37a696c)
- enhance API documentation for authentication endpoints (acdf3d1)
- enhance OpenAPI attributes and improve controller documentation (e27518d)

### Changed
- Add robots.txt and site.webmanifest for SEO and PWA support (d21bb8f)
- Implement code changes to enhance functionality and improve performance (57a33f2)

---
## v0.4.7 — 2026-07-16

### Fixed
- enhance release script and project generator for better version handling and documentation sync (a8e859e)

---
## v0.4.6 — 2026-07-16

### Added
- update version to 0.4.6 and enhance OpenAPI support with new attributes (f088ce6)

### Fixed
- comment out database module by default with setup guide (a0612d4)

---
## v0.4.5 — 2026-07-16

### Added

- `openapi` feature flag — OpenAPI/Swagger module is now feature-gated (was always compiled) and included in default features
- `logging` feature — structured time-series logging with `FileLogStorage` (`.logs/YYYY-MM-DD.jsonl`), `LogStorage` trait for pluggable backends, `TimeSeriesLayer` capturing all `tracing` events, and `ironic::log::{info, warn, error, debug, trace}` re-exports
- `logging` feature included in generated project template

### Fixed

- Generated project template now calls `.configure_router()` before `.with_openapi()` (method exists on `AxumAdapter`, not `OpenApiAxumAdapter`)
- Generated project now includes `sqlx` and `tracing` as direct dependencies for the database module
- `extern crate` aliases annotated with `#[allow(unused_extern_crates)]` to fix builds without default features
- Various code formatting fixes

---
## v0.4.4 — 2026-07-16

### Added
- enhance update command to automatically upgrade to the latest version (24228b6)

---
## v0.4.3 — 2026-07-16

### Fixed
- update default server host to 0.0.0.0 in multiple examples (435807c)
- update latest version in BlogIndex to v0.4.2 (2ca67ef)

---
## v0.4.2 — 2026-07-16

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)
- remove redundant command for cleaning stale test cache artifacts (e560244)
- update release script to check if version is published on crates.io before proceeding (d188dfc)

### Changed
- enhance getting started guide with project structure details (eb6ebeb)

---
## v0.4.1 — 2026-07-15

### Added
- add repository generation support in CLI and refactor todo app (09f74f4)
- Add comprehensive documentation for Todo API, database migrations, schema, architecture, deployment, and development setup (5034e24)
- initialize todo application with Ironic framework (4b19726)
- Enhance database integration documentation with setup instructions and examples (afea150)
- Add S3 upload documentation and update meta.json to include new page (630047e)
- Add configuration and migrations metadata, update advanced pages (16d2473)
- Update blog post for v0.4.0 with production readiness and enterprise features (b5790de)
- Update release notes for v0.4.0 with detailed features and improvements (336c954)
- Refactor imports in error and lib modules for better organization (199bc4f)

### Fixed
- Update configuration file names in tests for consistency (cc98918)
- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)
- Update actions/checkout version to v5 in CI workflow (e4c9e5d)
- Clean stale cache artifacts in CI workflow (56a9b2c)
- Remove redundant import and reorganize imports for clarity (1a4349d)

### Changed
- streamline code structure and improve readability across multiple files (3b7b0a2)

---
## v0.4.0 — 2026-07-15

### Added
- Implement production readiness improvements for Ironic (2bf4555)
- Add ready-resource generator for production-grade authentication module (ea28f4c)
- Add production readiness improvements across multiple components (948341b)
- add blog post on lifecycle hooks in axum integration (805a566)
- add blog posts on OnceCell-based singletons, sagas, scope violations, static plugin system, and two-phase route compilation (de3126e)
- refactor blog and releases index update logic in release script (8102c9a)
- update release notes and automate blog post generation for v0.3.9 (cb654ba)
- update changelog and release notes for v0.3.9 (699a8d6)
- add release notes for v0.3.9 and enhance release script documentation (08592c9)
- enhance release script to create blog post and update releases documentation (66b0a0a)

### Fixed
- update background styles in BlogIndex and BlogPage components (82f3c58)

### Changed
- Add new blog posts on various Ironic features and improvements (04a9ae9)
- Add blog posts on handler dispatch, injectable generation, and feature flags (fb37128)

---
