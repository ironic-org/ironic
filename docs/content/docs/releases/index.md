---
title: Releases
description: Version history and release notes for the Ironic framework.
---

# Releases

## Current version: v0.4.9

All notable changes to Ironic are documented here. The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

| Version | Date | Highlights |
|---------|------|-----------|
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
| [v0.1.x–v0.2.x](/blog/v0.1.x-v0.2.x) | 2026-07-14 | Pre-release journey — 19 iterations, auth, file upload, email modules |
| [v0.3.2](/blog/v0.3.0) | 2026-07-15 | Documentation nav fixes |
| [v0.3.1](/blog/v0.3.0) | 2026-07-15 | Ready-resource generator fixes |

Full changelog: [CHANGELOG.md](https://github.com/ironic-org/ironic/blob/main/CHANGELOG.md)

## Versioning policy

- **Major** (0.x → 1.0): Breaking API changes, significant design shifts
- **Minor** (0.3 → 0.4): New features, non-breaking additions
- **Patch** (0.3.7 → 0.3.8): Bug fixes, docs, small improvements
