---
title: v1.0.x
description: Complete changelog and release notes for the Ironic v1.0.x stable series.
---

# v1.0.x — Current Stable Series

All versions in the v1.0.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v1.0.3 — 2026-07-17

# v1.0.3

### Added
- add async test macro to simplify testing without external dependencies (5b03b7f)\n
### Fixed
- remove duplicate entry for 'r#test' in public use declarations (7491b26)\n- remove workflow_run trigger from release workflow to simplify event handling (0302fe6)\n- remove push event from CI workflow to streamline triggers (13f19fb)\n- update CI workflow to ignore specific paths on push and pull request events (869728d)\n- update CURRENT_VERSION to 1.0.2 (ce80e1b)\n- update CURRENT_VERSION to 1.0.1 (6802169)\n
### Changed
- update PR template and README to enforce code style and testing requirements (e8418cb)\n- enhance contributing guidelines with setup, branch naming, and commit message formats (ccf11cf)\n- Update issue templates (1f33ad5)\n

---
## v1.0.2 — 2026-07-17

# v1.0.2

### Added
- enhance release workflow with version detection and conditional execution (f79b4db)\n- auto-release pipeline — CI detects version bumps and triggers release (eea9041)\n
### Fixed
- update cargo publish commands to use env for CARGO_REGISTRY_TOKEN (aaa1b68)\n- maybe-release job needs actions:write permission to trigger release workflow (02c0dee)\n- release workflow now safe — tag only created by CI after publish (fde50af)\n

---
## v1.0.1 — 2026-07-17

# v1.0.1

### Added
- single version source of truth in docs/lib/constants.ts (0f01d78)\n- integrate bun for dependency management and build process in CI and release workflows (e975249)\n
### Fixed
- update caching keys for cargo-audit and cargo-deny in CI configuration (69c0244)\n

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
