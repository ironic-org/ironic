---
title: Releases
description: Version history and release notes for the Ironic framework.
---

# Releases

## Current version: v1.0.0

All notable changes to Ironic are documented here. The project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

| Version | Date | Highlights |
|---------|------|-----------|
| [v1.0.0](/blog/v1.0.0) | 2026-07-17 | Production release — CI/CD pipeline, matrix testing, crates.io publishing, production release guide, blog-api example, and 200+ tests |

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
