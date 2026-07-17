---
title: v1.0.x
description: Complete changelog and release notes for the Ironic v1.0.x stable series.
---

# v1.0.x — Current Stable Series

All versions in the v1.0.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v1.0.0 — 2026-07-17

# v1.0.0

After extensive development through the 0.x series, Ironic 1.0.0 marks the framework's first stable production release. This release focuses on the operational and procedural aspects of production software — CI/CD, security auditing, release automation, and documentation.

### Added

- GitHub Actions CI with matrix testing across stable and nightly Rust (e3e863c)
- Separate security job for cargo audit and cargo deny checks (4872ed8)
- crates.io publishing workflow in release pipeline — publishes ironic-macros first, then ironic (e3e863c)
- Fuzz testing job (60s smoke check with cargo-fuzz on nightly) (4872ed8)
- Production release guide with pre-flight checklist, versioning policy, hotfix process, and rollback plan (e3e863c)
- Blog API example demonstrating cross-module dependency injection, category management, slug generation, and stats module (e3e863c)
- SECURITY.md updated for 1.0.x supported versions (4872ed8)
- Dependabot configuration for automated dependency updates (e3e863c)

### Changed

- Version bumped from 0.4.9 to 1.0.0 (e3e863c)
- CI workflow restructured with separate check, security, and fuzz jobs for faster feedback (4872ed8)
- Releases index updated with 1.0 versioning policy and strict SemVer adherence (e3e863c)
- All documentation references updated from 0.4.x to 1.0.0 (4872ed8)
- v0.5.x series marked as legacy in releases documentation (e3e863c)

### Fixed

- Release workflow now caches cargo-deny and cargo-audit binaries for faster runs (4872ed8)
- Getting-started docs show correct version number (1.0.0) in CLI examples (e3e863c)

---
