---
title: v0.4.x
description: Complete changelog and release notes for the Ironic v0.4.x stable series.
---

# v0.4.x — Current Stable Series

All versions in the v0.4.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v0.4.2 — July 16, 2026

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)
- remove redundant command for cleaning stale test cache artifacts (e560244)
- update release script to check if version is published on crates.io before proceeding (d188dfc)

### Changed
- release v0.4.2 with updates and fixes (226ef26)
- enhance getting started guide with project structure details (eb6ebeb)

---

## v0.4.1 — July 15, 2026

### Added
- Enhance production readiness with telemetry, metrics, health checks, and more (3feff61)
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
- update path for ironic dependency in todo app example (18d76f0)
- Update configuration file names in tests for consistency (cc98918)
- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)
- Update actions/checkout version to v5 in CI workflow (e4c9e5d)
- Clean stale cache artifacts in CI workflow (56a9b2c)
- Remove redundant import and reorganize imports for clarity (1a4349d)

### Changed
- remove unused dependencies and example from Cargo configuration (5a2dcf0)
- release v0.4.1 with new features, improvements, and documentation updates (4784cdb)
- streamline code structure and improve readability across multiple files (3b7b0a2)

---

## v0.4.0 — July 15, 2026

### Added
- Update release script to bump current version and add new release to index (481aa56)
- Update release script to publish to crates.io before git push (7e1ad7a)
- Update changelog and add blog post for v0.4.0 release (dbd6837)
- Update telemetry and tracing documentation (fc34232)
- Implement production readiness improvements for Ironic (2bf455c)
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
- Correct Prometheus output format code block in metrics documentation (1475744)
- update background styles in BlogIndex and BlogPage components (82f3c58)

### Changed
- Add new blog posts on various Ironic features and improvements (04a9ae9)
- Add blog posts on handler dispatch, injectable generation, and feature flags (fb37128)

---

## v0.3.9 — Legacy

