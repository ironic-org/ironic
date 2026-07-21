---
title: v1.0.x
description: Complete changelog and release notes for the Ironic v1.0.x stable series.
---

# v1.0.x — Current Stable Series

All versions in the v1.0.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v1.0.9 — 2026-07-21

# v1.0.9

### Added
- add documentation for backtrace and UUID features, and implement message queues and saga orchestration (53406ae)
- enhance observability with ISO timestamp and refined logging duration (7102c19)
- update SQLx version to 0.9 and enhance feature flag documentation (7288388)
- update CURRENT_VERSION to 1.0.8 and clean up unused imports in lib.rs (34ad25d)
- add pagination extractor and SQL error mapping utilities (7b4fcdc)
- implement blog module with CRUD operations for blog posts and categories (ec5e067)
- implement blog API example with CRUD operations and JWT authentication (a2f68ca)
- add lifecycle hooks for application and module management (3f7e160)
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)
- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)
- add new lifecycle hooks and enhance existing ones (faff30a)
- Add global exception middleware for improved error handling (3d439ed)
- Implement authentication module with JWT support (775894b)
- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)
- update middleware documentation and structure, add new custom middleware section (3fd54be)
- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)
- add saas-starter-kit to .gitignore (67d9330)
- update funding information and add sponsorship section to README (e170210)
- add uninstall command to remove Ironic binary and caches (de9df21)
- add FormBody extractor and #[form] attribute (1c468ac)

### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)

### Changed
- remove Dependabot configuration file (1656fa3)
- release v1.0.8 (e1e0824)
- cc (4dee27a)
- release v1.0.7 (bea60af)
- release v1.0.6 (98c3050)
- release v1.0.5 (3ecf491)
- remove unused example project from workspace members (6f465ca)
- example project (e664847)
- Remove todo-app example project files and related documentation (d5409ee)
- update logo and favicon to SVG format for better scalability (e113e5e)
- enhance comparison table with additional features and details (3f8d749)
- release v1.0.5 (198fbc2)
- release v1.0.4 (c50b23e)
- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)
- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)
- update request type in custom decorator examples to use Request (4593c67)
- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)
- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)
- Refactor FrameworkApplication to Application (be9da2e)
- Add documentation for new features and modules (93cea95)
- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)
- remove NotFoundFilter and update documentation for exception handling (d228045)
- Replace serde_json with ironic::json in blog-api module (4b61ec6)
- Replace tracing with ironic logging in blog-api module (0ee1b79)
- Enhance middleware documentation and features (ae38d8e)
- release v1.0.4 (c953e8d)

---
## v1.0.8 — 2026-07-18

# v1.0.8

### Added
- add pagination extractor and SQL error mapping utilities (7b4fcdc)
- implement blog module with CRUD operations for blog posts and categories (ec5e067)
- implement blog API example with CRUD operations and JWT authentication (a2f68ca)
- add lifecycle hooks for application and module management (3f7e160)
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)
- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)
- add new lifecycle hooks and enhance existing ones (faff30a)
- Add global exception middleware for improved error handling (3d439ed)
- Implement authentication module with JWT support (775894b)
- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)
- update middleware documentation and structure, add new custom middleware section (3fd54be)
- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)
- add saas-starter-kit to .gitignore (67d9330)
- update funding information and add sponsorship section to README (e170210)
- add uninstall command to remove Ironic binary and caches (de9df21)
- add FormBody extractor and #[form] attribute (1c468ac)

### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)

### Changed
- cc (4dee27a)
- release v1.0.7 (bea60af)
- release v1.0.6 (98c3050)
- release v1.0.5 (3ecf491)
- remove unused example project from workspace members (6f465ca)
- example project (e664847)
- Remove todo-app example project files and related documentation (d5409ee)
- update logo and favicon to SVG format for better scalability (e113e5e)
- enhance comparison table with additional features and details (3f8d749)
- release v1.0.5 (198fbc2)
- release v1.0.4 (c50b23e)
- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)
- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)
- update request type in custom decorator examples to use Request (4593c67)
- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)
- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)
- Refactor FrameworkApplication to Application (be9da2e)
- Add documentation for new features and modules (93cea95)
- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)
- remove NotFoundFilter and update documentation for exception handling (d228045)
- Replace serde_json with ironic::json in blog-api module (4b61ec6)
- Replace tracing with ironic logging in blog-api module (0ee1b79)
- Enhance middleware documentation and features (ae38d8e)
- release v1.0.4 (c953e8d)

---
## v1.0.7 — 2026-07-18

# v1.0.7

### Added
- add pagination extractor and SQL error mapping utilities (7b4fcdc)
- implement blog module with CRUD operations for blog posts and categories (ec5e067)
- implement blog API example with CRUD operations and JWT authentication (a2f68ca)
- add lifecycle hooks for application and module management (3f7e160)
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)
- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)
- add new lifecycle hooks and enhance existing ones (faff30a)
- Add global exception middleware for improved error handling (3d439ed)
- Implement authentication module with JWT support (775894b)
- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)
- update middleware documentation and structure, add new custom middleware section (3fd54be)
- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)
- add saas-starter-kit to .gitignore (67d9330)
- update funding information and add sponsorship section to README (e170210)
- add uninstall command to remove Ironic binary and caches (de9df21)
- add FormBody extractor and #[form] attribute (1c468ac)

### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)

### Changed
- release v1.0.6 (98c3050)
- release v1.0.5 (3ecf491)
- remove unused example project from workspace members (6f465ca)
- example project (e664847)
- Remove todo-app example project files and related documentation (d5409ee)
- update logo and favicon to SVG format for better scalability (e113e5e)
- enhance comparison table with additional features and details (3f8d749)
- release v1.0.5 (198fbc2)
- release v1.0.4 (c50b23e)
- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)
- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)
- update request type in custom decorator examples to use Request (4593c67)
- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)
- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)
- Refactor FrameworkApplication to Application (be9da2e)
- Add documentation for new features and modules (93cea95)
- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)
- remove NotFoundFilter and update documentation for exception handling (d228045)
- Replace serde_json with ironic::json in blog-api module (4b61ec6)
- Replace tracing with ironic logging in blog-api module (0ee1b79)
- Enhance middleware documentation and features (ae38d8e)
- release v1.0.4 (c953e8d)

---
## v1.0.6 — 2026-07-18

# v1.0.6

### Added
- implement blog API example with CRUD operations and JWT authentication (a2f68ca)
- add lifecycle hooks for application and module management (3f7e160)
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)
- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)
- add new lifecycle hooks and enhance existing ones (faff30a)
- Add global exception middleware for improved error handling (3d439ed)
- Implement authentication module with JWT support (775894b)
- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)
- update middleware documentation and structure, add new custom middleware section (3fd54be)
- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)
- add saas-starter-kit to .gitignore (67d9330)
- update funding information and add sponsorship section to README (e170210)
- add uninstall command to remove Ironic binary and caches (de9df21)
- add FormBody extractor and #[form] attribute (1c468ac)

### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)

### Changed
- release v1.0.5 (3ecf491)
- remove unused example project from workspace members (6f465ca)
- example project (e664847)
- Remove todo-app example project files and related documentation (d5409ee)
- update logo and favicon to SVG format for better scalability (e113e5e)
- enhance comparison table with additional features and details (3f8d749)
- release v1.0.5 (198fbc2)
- release v1.0.4 (c50b23e)
- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)
- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)
- update request type in custom decorator examples to use Request (4593c67)
- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)
- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)
- Refactor FrameworkApplication to Application (be9da2e)
- Add documentation for new features and modules (93cea95)
- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)
- remove NotFoundFilter and update documentation for exception handling (d228045)
- Replace serde_json with ironic::json in blog-api module (4b61ec6)
- Replace tracing with ironic logging in blog-api module (0ee1b79)
- Enhance middleware documentation and features (ae38d8e)
- release v1.0.4 (c953e8d)

---
## v1.0.5 — 2026-07-18

# v1.0.5

### Added
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)
- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)
- add new lifecycle hooks and enhance existing ones (faff30a)
- Add global exception middleware for improved error handling (3d439ed)
- Implement authentication module with JWT support (775894b)
- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)
- update middleware documentation and structure, add new custom middleware section (3fd54be)
- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)
- add saas-starter-kit to .gitignore (67d9330)
- update funding information and add sponsorship section to README (e170210)
- add uninstall command to remove Ironic binary and caches (de9df21)
- add FormBody extractor and #[form] attribute (1c468ac)

### Fixed
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)

### Changed
- release v1.0.4 (c50b23e)
- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)
- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)
- update request type in custom decorator examples to use Request (4593c67)
- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)
- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)
- Refactor FrameworkApplication to Application (be9da2e)
- Add documentation for new features and modules (93cea95)
- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)
- remove NotFoundFilter and update documentation for exception handling (d228045)
- Replace serde_json with ironic::json in blog-api module (4b61ec6)
- Replace tracing with ironic logging in blog-api module (0ee1b79)
- Enhance middleware documentation and features (ae38d8e)
- release v1.0.4 (c953e8d)

---
## v1.0.4 — 2026-07-17

# v1.0.4

### Added
- add async test macro to simplify testing without external dependencies (5b03b7f)
- add workflow documentation for CI/CD release process (a9ccd2e)

### Fixed
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)
- remove duplicate entry for 'r#test' in public use declarations (7491b26)
- remove workflow_run trigger from release workflow to simplify event handling (0302fe6)
- remove push event from CI workflow to streamline triggers (13f19fb)
- update CI workflow to ignore specific paths on push and pull request events (869728d)
- update CURRENT_VERSION to 1.0.2 (ce80e1b)
- update CURRENT_VERSION to 1.0.1 (6802169)

### Changed
- release v1.0.3 (051991f)
- update PR template and README to enforce code style and testing requirements (e8418cb)
- enhance contributing guidelines with setup, branch naming, and commit message formats (ccf11cf)
- Update issue templates (1f33ad5)

---
## v1.0.3 — 2026-07-17

# v1.0.3

### Added
- add async test macro to simplify testing without external dependencies (5b03b7f)

### Fixed
- remove duplicate entry for 'r#test' in public use declarations (7491b26)
- remove workflow_run trigger from release workflow to simplify event handling (0302fe6)
- remove push event from CI workflow to streamline triggers (13f19fb)
- update CI workflow to ignore specific paths on push and pull request events (869728d)
- update CURRENT_VERSION to 1.0.2 (ce80e1b)
- update CURRENT_VERSION to 1.0.1 (6802169)

### Changed
- update PR template and README to enforce code style and testing requirements (e8418cb)
- enhance contributing guidelines with setup, branch naming, and commit message formats (ccf11cf)
- Update issue templates (1f33ad5)

---
## v1.0.2 — 2026-07-17

# v1.0.2

### Added
- enhance release workflow with version detection and conditional execution (f79b4db)
- auto-release pipeline — CI detects version bumps and triggers release (eea9041)

### Fixed
- update cargo publish commands to use env for CARGO_REGISTRY_TOKEN (aaa1b68)
- maybe-release job needs actions:write permission to trigger release workflow (02c0dee)
- release workflow now safe — tag only created by CI after publish (fde50af)

---
## v1.0.1 — 2026-07-17

# v1.0.1

### Added
- single version source of truth in docs/lib/constants.ts (0f01d78)
- integrate bun for dependency management and build process in CI and release workflows (e975249)

### Fixed
- update caching keys for cargo-audit and cargo-deny in CI configuration (69c0244)

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
