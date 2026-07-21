# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- release.sh: prefer [Unreleased] content over git log when non-empty
- add-changelog-entry.sh: helper script for quick [Unreleased] entries

### Fixed
- release.sh: macOS compat — replace head -n -1 with sed '$d'
- add-changelog-entry.sh: handle pipefail grep exits with || true
- add-changelog-entry.sh: prevent duplicate category insertion

## [v1.0.9] - 2026-07-21

### Added
- add documentation for backtrace and UUID features, and implement message queues and saga orchestration (53406ae)\n- enhance observability with ISO timestamp and refined logging duration (7102c19)\n- update SQLx version to 0.9 and enhance feature flag documentation (7288388)\n- update CURRENT_VERSION to 1.0.8 and clean up unused imports in lib.rs (34ad25d)\n- add pagination extractor and SQL error mapping utilities (7b4fcdc)\n- implement blog module with CRUD operations for blog posts and categories (ec5e067)\n- implement blog API example with CRUD operations and JWT authentication (a2f68ca)\n- add lifecycle hooks for application and module management (3f7e160)\n- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)\n- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)\n- add new lifecycle hooks and enhance existing ones (faff30a)\n- Add global exception middleware for improved error handling (3d439ed)\n- Implement authentication module with JWT support (775894b)\n- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)\n- update middleware documentation and structure, add new custom middleware section (3fd54be)\n- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)\n- add saas-starter-kit to .gitignore (67d9330)\n- update funding information and add sponsorship section to README (e170210)\n- add uninstall command to remove Ironic binary and caches (de9df21)\n- add FormBody extractor and #[form] attribute (1c468ac)\n
### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)\n- update response body mapping in platform adapter documentation (e63a720)\n- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n
### Changed
- remove Dependabot configuration file (1656fa3)\n- release v1.0.8 (e1e0824)\n- cc (4dee27a)\n- release v1.0.7 (bea60af)\n- release v1.0.6 (98c3050)\n- release v1.0.5 (3ecf491)\n- remove unused example project from workspace members (6f465ca)\n- example project (e664847)\n- Remove todo-app example project files and related documentation (d5409ee)\n- update logo and favicon to SVG format for better scalability (e113e5e)\n- enhance comparison table with additional features and details (3f8d749)\n- release v1.0.5 (198fbc2)\n- release v1.0.4 (c50b23e)\n- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)\n- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)\n- update request type in custom decorator examples to use Request (4593c67)\n- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)\n- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)\n- Refactor FrameworkApplication to Application (be9da2e)\n- Add documentation for new features and modules (93cea95)\n- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)\n- remove NotFoundFilter and update documentation for exception handling (d228045)\n- Replace serde_json with ironic::json in blog-api module (4b61ec6)\n- Replace tracing with ironic logging in blog-api module (0ee1b79)\n- Enhance middleware documentation and features (ae38d8e)\n- release v1.0.4 (c953e8d)\n

## [v1.0.8] - 2026-07-18

### Added
- add pagination extractor and SQL error mapping utilities (7b4fcdc)\n- implement blog module with CRUD operations for blog posts and categories (ec5e067)\n- implement blog API example with CRUD operations and JWT authentication (a2f68ca)\n- add lifecycle hooks for application and module management (3f7e160)\n- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)\n- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)\n- add new lifecycle hooks and enhance existing ones (faff30a)\n- Add global exception middleware for improved error handling (3d439ed)\n- Implement authentication module with JWT support (775894b)\n- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)\n- update middleware documentation and structure, add new custom middleware section (3fd54be)\n- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)\n- add saas-starter-kit to .gitignore (67d9330)\n- update funding information and add sponsorship section to README (e170210)\n- add uninstall command to remove Ironic binary and caches (de9df21)\n- add FormBody extractor and #[form] attribute (1c468ac)\n
### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)\n- update response body mapping in platform adapter documentation (e63a720)\n- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n
### Changed
- cc (4dee27a)\n- release v1.0.7 (bea60af)\n- release v1.0.6 (98c3050)\n- release v1.0.5 (3ecf491)\n- remove unused example project from workspace members (6f465ca)\n- example project (e664847)\n- Remove todo-app example project files and related documentation (d5409ee)\n- update logo and favicon to SVG format for better scalability (e113e5e)\n- enhance comparison table with additional features and details (3f8d749)\n- release v1.0.5 (198fbc2)\n- release v1.0.4 (c50b23e)\n- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)\n- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)\n- update request type in custom decorator examples to use Request (4593c67)\n- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)\n- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)\n- Refactor FrameworkApplication to Application (be9da2e)\n- Add documentation for new features and modules (93cea95)\n- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)\n- remove NotFoundFilter and update documentation for exception handling (d228045)\n- Replace serde_json with ironic::json in blog-api module (4b61ec6)\n- Replace tracing with ironic logging in blog-api module (0ee1b79)\n- Enhance middleware documentation and features (ae38d8e)\n- release v1.0.4 (c953e8d)\n

## [v1.0.7] - 2026-07-18

### Added
- add pagination extractor and SQL error mapping utilities (7b4fcdc)\n- implement blog module with CRUD operations for blog posts and categories (ec5e067)\n- implement blog API example with CRUD operations and JWT authentication (a2f68ca)\n- add lifecycle hooks for application and module management (3f7e160)\n- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)\n- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)\n- add new lifecycle hooks and enhance existing ones (faff30a)\n- Add global exception middleware for improved error handling (3d439ed)\n- Implement authentication module with JWT support (775894b)\n- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)\n- update middleware documentation and structure, add new custom middleware section (3fd54be)\n- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)\n- add saas-starter-kit to .gitignore (67d9330)\n- update funding information and add sponsorship section to README (e170210)\n- add uninstall command to remove Ironic binary and caches (de9df21)\n- add FormBody extractor and #[form] attribute (1c468ac)\n
### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)\n- update response body mapping in platform adapter documentation (e63a720)\n- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n
### Changed
- release v1.0.6 (98c3050)\n- release v1.0.5 (3ecf491)\n- remove unused example project from workspace members (6f465ca)\n- example project (e664847)\n- Remove todo-app example project files and related documentation (d5409ee)\n- update logo and favicon to SVG format for better scalability (e113e5e)\n- enhance comparison table with additional features and details (3f8d749)\n- release v1.0.5 (198fbc2)\n- release v1.0.4 (c50b23e)\n- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)\n- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)\n- update request type in custom decorator examples to use Request (4593c67)\n- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)\n- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)\n- Refactor FrameworkApplication to Application (be9da2e)\n- Add documentation for new features and modules (93cea95)\n- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)\n- remove NotFoundFilter and update documentation for exception handling (d228045)\n- Replace serde_json with ironic::json in blog-api module (4b61ec6)\n- Replace tracing with ironic logging in blog-api module (0ee1b79)\n- Enhance middleware documentation and features (ae38d8e)\n- release v1.0.4 (c953e8d)\n

## [v1.0.6] - 2026-07-18

### Added
- implement blog API example with CRUD operations and JWT authentication (a2f68ca)\n- add lifecycle hooks for application and module management (3f7e160)\n- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)\n- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)\n- add new lifecycle hooks and enhance existing ones (faff30a)\n- Add global exception middleware for improved error handling (3d439ed)\n- Implement authentication module with JWT support (775894b)\n- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)\n- update middleware documentation and structure, add new custom middleware section (3fd54be)\n- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)\n- add saas-starter-kit to .gitignore (67d9330)\n- update funding information and add sponsorship section to README (e170210)\n- add uninstall command to remove Ironic binary and caches (de9df21)\n- add FormBody extractor and #[form] attribute (1c468ac)\n
### Fixed
- add winnow duplicate to cargo-deny skip list (bd85f3f)\n- update response body mapping in platform adapter documentation (e63a720)\n- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n
### Changed
- release v1.0.5 (3ecf491)\n- remove unused example project from workspace members (6f465ca)\n- example project (e664847)\n- Remove todo-app example project files and related documentation (d5409ee)\n- update logo and favicon to SVG format for better scalability (e113e5e)\n- enhance comparison table with additional features and details (3f8d749)\n- release v1.0.5 (198fbc2)\n- release v1.0.4 (c50b23e)\n- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)\n- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)\n- update request type in custom decorator examples to use Request (4593c67)\n- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)\n- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)\n- Refactor FrameworkApplication to Application (be9da2e)\n- Add documentation for new features and modules (93cea95)\n- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)\n- remove NotFoundFilter and update documentation for exception handling (d228045)\n- Replace serde_json with ironic::json in blog-api module (4b61ec6)\n- Replace tracing with ironic logging in blog-api module (0ee1b79)\n- Enhance middleware documentation and features (ae38d8e)\n- release v1.0.4 (c953e8d)\n

## [v1.0.5] - 2026-07-18

### Added
- implement feature gate guard for runtime feature toggling and enhance lifecycle hooks with module load/unload callbacks (ad83aaf)\n- add ExceptionExt trait for inline exception handling and update documentation (95d85a0)\n- add new lifecycle hooks and enhance existing ones (faff30a)\n- Add global exception middleware for improved error handling (3d439ed)\n- Implement authentication module with JWT support (775894b)\n- add VITE_GIT_BRANCH to environment and display in UI components (6b6929c)\n- update middleware documentation and structure, add new custom middleware section (3fd54be)\n- add RequestLogging middleware for structured HTTP request/response logging (3ed0763)\n- add saas-starter-kit to .gitignore (67d9330)\n- update funding information and add sponsorship section to README (e170210)\n- add uninstall command to remove Ironic binary and caches (de9df21)\n- add FormBody extractor and #[form] attribute (1c468ac)\n
### Fixed
- update response body mapping in platform adapter documentation (e63a720)\n- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n
### Changed
- release v1.0.4 (c50b23e)\n- update documentation and code to replace 'Framework' with 'Ironic' (a7e617c)\n- replace Framework types with simplified Request and Response across documentation and code (b1fd59d)\n- update request type in custom decorator examples to use Request (4593c67)\n- replace FrameworkBody with Body in response serialization and streaming documentation (3e71713)\n- rename FrameworkRequest and FrameworkResponse to Request and Response (e7a6928)\n- Refactor FrameworkApplication to Application (be9da2e)\n- Add documentation for new features and modules (93cea95)\n- update documentation for WebSocket message handlers, interceptors, and feature flags; add operational endpoints to observability (01bbcb7)\n- remove NotFoundFilter and update documentation for exception handling (d228045)\n- Replace serde_json with ironic::json in blog-api module (4b61ec6)\n- Replace tracing with ironic logging in blog-api module (0ee1b79)\n- Enhance middleware documentation and features (ae38d8e)\n- release v1.0.4 (c953e8d)\n

## [v1.0.4] - 2026-07-17

### Added
- add async test macro to simplify testing without external dependencies (5b03b7f)\n- add workflow documentation for CI/CD release process (a9ccd2e)\n
### Fixed
- handle missing environment variables in BuildInfo (39e64f1)\n- update CURRENT_VERSION to 1.0.3 (42468f6)\n- remove duplicate entry for 'r#test' in public use declarations (7491b26)\n- remove workflow_run trigger from release workflow to simplify event handling (0302fe6)\n- remove push event from CI workflow to streamline triggers (13f19fb)\n- update CI workflow to ignore specific paths on push and pull request events (869728d)\n- update CURRENT_VERSION to 1.0.2 (ce80e1b)\n- update CURRENT_VERSION to 1.0.1 (6802169)\n
### Changed
- release v1.0.3 (051991f)\n- update PR template and README to enforce code style and testing requirements (e8418cb)\n- enhance contributing guidelines with setup, branch naming, and commit message formats (ccf11cf)\n- Update issue templates (1f33ad5)\n

## [v1.0.3] - 2026-07-17

### Added
- add async test macro to simplify testing without external dependencies (5b03b7f)\n
### Fixed
- remove duplicate entry for 'r#test' in public use declarations (7491b26)\n- remove workflow_run trigger from release workflow to simplify event handling (0302fe6)\n- remove push event from CI workflow to streamline triggers (13f19fb)\n- update CI workflow to ignore specific paths on push and pull request events (869728d)\n- update CURRENT_VERSION to 1.0.2 (ce80e1b)\n- update CURRENT_VERSION to 1.0.1 (6802169)\n
### Changed
- update PR template and README to enforce code style and testing requirements (e8418cb)\n- enhance contributing guidelines with setup, branch naming, and commit message formats (ccf11cf)\n- Update issue templates (1f33ad5)\n

## [v1.0.2] - 2026-07-17

### Added
- enhance release workflow with version detection and conditional execution (f79b4db)\n- auto-release pipeline — CI detects version bumps and triggers release (eea9041)\n
### Fixed
- update cargo publish commands to use env for CARGO_REGISTRY_TOKEN (aaa1b68)\n- maybe-release job needs actions:write permission to trigger release workflow (02c0dee)\n- release workflow now safe — tag only created by CI after publish (fde50af)\n

## [v1.0.1] - 2026-07-17

### Added
- single version source of truth in docs/lib/constants.ts (0f01d78)\n- integrate bun for dependency management and build process in CI and release workflows (e975249)\n
### Fixed
- update caching keys for cargo-audit and cargo-deny in CI configuration (69c0244)\n

## [v1.0.0] - 2026-07-17

### Added
- GitHub Actions CI with matrix testing across stable and nightly Rust (e3e863c)
- Separate security job for cargo audit and cargo deny checks (4872ed8)
- crates.io publishing workflow in release pipeline (e3e863c)
- Fuzz testing job (60s smoke check with cargo-fuzz on nightly) (4872ed8)
- Production release guide with pre-flight checklist, versioning policy, hotfix process, and rollback plan (e3e863c)
- Blog API example demonstrating cross-module DI, categories, slug management, and stats (e3e863c)
- SECURITY.md updated for 1.0.x supported versions (4872ed8)
- Dependabot configuration for automated dependency updates (e3e863c)

### Changed
- Version bumped from 0.4.9 to 1.0.0 (e3e863c)
- CI workflow restructured with separate check, security, and fuzz jobs (4872ed8)
- Releases index updated with 1.0 versioning policy (e3e863c)
- All documentation references updated to 1.0.0 (4872ed8)
- v0.5.x series marked as legacy (e3e863c)

## [v0.5.0] - 2026-07-16

### Fixed
- update alias for Decorator command from 'd' to 'de' (e3e863c)\n- update npm command in CI workflow to use 'install' instead of 'ci' (4872ed8)\n

## [v0.4.9] - 2026-07-16

### Added
- implement CI/CD pipeline, security auditing, and operational endpoints (e5537f2)\n- enhance observability with operational endpoints and health checks (0082bdb)\n
### Fixed
- improve documentation and formatting in build script and tests (5226611)\n

## [v0.4.8] - 2026-07-16

### Added
- add database migration commands and update documentation (1e3db79)\n
### Fixed
- improve formatting and readability in migration and project generation code (37a696c)\n- enhance API documentation for authentication endpoints (acdf3d1)\n- enhance OpenAPI attributes and improve controller documentation (e27518d)\n
### Changed
- Add robots.txt and site.webmanifest for SEO and PWA support (d21bb8f)\n- Implement code changes to enhance functionality and improve performance (57a33f2)\n

## [v0.4.7] - 2026-07-16

### Fixed
- enhance release script and project generator for better version handling and documentation sync (a8e859e)\n

## [v0.4.6] - 2026-07-16

### Added
- update version to 0.4.6 and enhance OpenAPI support with new attributes (f088ce6)\n
### Fixed
- comment out database module by default with setup guide (a0612d4)\n

## [v0.4.5] - 2026-07-16

### Added

- `openapi` feature flag — OpenAPI/Swagger module is now feature-gated (was always compiled) and included in default features
- `logging` feature — structured time-series logging with `FileLogStorage` (`.logs/YYYY-MM-DD.jsonl`), `LogStorage` trait for pluggable backends, `TimeSeriesLayer` capturing all `tracing` events, and `ironic::log::{info, warn, error, debug, trace}` re-exports
- `logging` feature included in generated project template

### Fixed

- Generated project template now calls `.configure_router()` before `.with_openapi()` (method exists on `AxumAdapter`, not `OpenApiAxumAdapter`)
- Generated project now includes `sqlx` and `tracing` as direct dependencies for the database module
- `extern crate` aliases annotated with `#[allow(unused_extern_crates)]` to fix builds without default features
- Various code formatting fixes

## [v0.4.4] - 2026-07-16

### Added
- enhance update command to automatically upgrade to the latest version (24228b6)\n

## [v0.4.3] - 2026-07-16

### Fixed
- update default server host to 0.0.0.0 in multiple examples (435807c)\n- update latest version in BlogIndex to v0.4.2 (2ca67ef)\n

## [v0.4.2] - 2026-07-16

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)\n- remove redundant command for cleaning stale test cache artifacts (e560244)\n- update release script to check if version is published on crates.io before proceeding (d188dfc)\n
### Changed
- enhance getting started guide with project structure details (eb6ebeb)\n

## [v0.4.1] - 2026-07-15

### Added
- add repository generation support in CLI and refactor todo app (09f74f4)\n- Add comprehensive documentation for Todo API, database migrations, schema, architecture, deployment, and development setup (5034e24)\n- initialize todo application with Ironic framework (4b19726)\n- Enhance database integration documentation with setup instructions and examples (afea150)\n- Add S3 upload documentation and update meta.json to include new page (630047e)\n- Add configuration and migrations metadata, update advanced pages (16d2473)\n- Update blog post for v0.4.0 with production readiness and enterprise features (b5790de)\n- Update release notes for v0.4.0 with detailed features and improvements (336c954)\n- Refactor imports in error and lib modules for better organization (199bc4f)\n
### Fixed
- Update configuration file names in tests for consistency (cc98918)\n- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)\n- Update actions/checkout version to v5 in CI workflow (e4c9e5d)\n- Clean stale cache artifacts in CI workflow (56a9b2c)\n- Remove redundant import and reorganize imports for clarity (1a4349d)\n
### Changed
- streamline code structure and improve readability across multiple files (3b7b0a2)\n

## [v0.4.0] - 2026-07-15

### Added
- Implement production readiness improvements for Ironic (2bf4555)\n- Add ready-resource generator for production-grade authentication module (ea28f4c)\n- Add production readiness improvements across multiple components (948341b)\n- add blog post on lifecycle hooks in axum integration (805a566)\n- add blog posts on OnceCell-based singletons, sagas, scope violations, static plugin system, and two-phase route compilation (de3126e)\n- refactor blog and releases index update logic in release script (8102c9a)\n- update release notes and automate blog post generation for v0.3.9 (cb654ba)\n- update changelog and release notes for v0.3.9 (699a8d6)\n- add release notes for v0.3.9 and enhance release script documentation (08592c9)\n- enhance release script to create blog post and update releases documentation (66b0a0a)\n
### Fixed
- update background styles in BlogIndex and BlogPage components (82f3c58)\n
### Changed
- Add new blog posts on various Ironic features and improvements (04a9ae9)\n- Add blog posts on handler dispatch, injectable generation, and feature flags (fb37128)\n

## [v0.3.9] - 2026-07-15

### Added
- add release notes for v0.3.9 and enhance release script documentation (08592c9)\n- enhance release script to create blog post and update releases documentation (66b0a0a)\n

## [v0.3.8] - 2026-07-15

### Added
- enhance observability section with health checks, metrics, and tracing documentation (cf2cc42)\n- update server host in dotenv example and Dockerfile for better accessibility (381f0eb)\n- update Dockerfile generation to use kebab-case project name (137202a)\n

## [v0.3.7] - 2026-07-15

### Added
- add global middleware support for application builder and enhance security features (7113eef)\n

## [v0.3.6] - 2026-07-15

### Added
- update validation pipes documentation with comprehensive examples and improved descriptions (c56dc5b)\n- add basic and auth API examples with CRUD functionality (b10e11e)\n- enhance project manifest with additional dependencies and security features (613d478)\n
### Fixed
- allow dead code warnings for unit tests in authentication module (77c5c02)\n
### Changed
- update version to 0.3.6 and remove unused API examples from workspace (914a74d)\n

## [v0.3.5] - 2026-07-15

### Fixed
- refactor authentication test file structure and update module imports (97720ac)\n

## [v0.3.4] - 2026-07-15

### Fixed
- remove unused integration module from tests (61aa525)\n- update integration test file paths for auth modules (db79152)\n- docs pages deployment with .nojekyll and SPA fallback (310efb2)\n

## [v0.3.3] - 2026-07-15

### Added
- auto-add required dependencies to Cargo.toml during module registration (e8de7ce)\n
### Fixed
- format manual instructions for clarity in module registration (4f55008)\n

## [v0.3.2] - 2026-07-15

### Fixed
- update documentation link in navigation component for clarity (d9eafaf)\n- update parameter names for consistency in auth module decorators and guards (18009e6)\n

## [v0.3.1] - 2026-07-15

### Fixed
- allow needless raw string hashes and restore GenerationReport import in ready_resource.rs (583ba86)\n
### Changed
- bump version to 0.3.1 in Cargo.toml and Cargo.lock (d4d7b20)\n- reorder module imports for consistency in ready_resource.rs (7fd6159)\n- update module imports and improve code readability in ready_resource.rs (d7d944f)\n

## [v0.3.0] - 2026-07-15

- Initial release


## [v0.2.9] - 2026-07-15

### Added
- update changelog and add new ready-resource documentation for authentication, file upload, and email modules (07f6232)\n- add file upload and email modules with respective generators (3bc21f8)\n- add comprehensive authentication module with various strategies (8dc08b2)\n- add ready-resource generator for complete authentication module with variants (81e9e9f)\n
### Fixed
- update error code reference in rate limit middleware (603fcae)\n- update permissions and restructure GitHub Actions workflow for documentation deployment (f63caf3)\n- add permissions section for GitHub Actions workflow to enable content writing (0800ae6)\n- adjust formatting of router creation in main.tsx for improved readability (e76ab60)\n- simplify GitHub Actions workflow for deploying documentation to GitHub Pages (5841216)\n- restructure GitHub Actions workflow for deploying documentation to GitHub Pages (1856566)\n- update link in HeroSection to point to the getting started page (0890f33)\n
### Changed
- simplify register_module function signature (5733b4f)\n

## [v0.2.8] - 2026-07-14

### Added
- update dotenv example with placeholder values and improve CSRF cookie/header name validation (da96fc8)\n
### Fixed
- handle poisoned mutex locks in metrics, resilience, security modules (399821a)\n
### Changed
- streamline CorsConfig initialization in tests (9517f27)\n- update CORS configuration tests to reflect default deny behavior and explicit origin allowance (90e16ad)\n

## [v0.2.7] - 2026-07-14

### Added
- add dotenvy support for configurable server host and port in main source (846e89b)\n

## [v0.2.6] - 2026-07-14

### Added
- improve changelog generation with formatted entries and enhanced parsing (fdcac78)\n- add changelog generation to release script (0653753)\n- enhance project scaffold generation with example module and CI workflow (13a29dc)\n
### Changed
- update version numbers to 0.2.5 in documentation and code (9408e57)\n

## [v0.2.5] - 2026-07-14nn### Addedn- feat: add changelog generation to release script (0653753)n- feat: enhance project scaffold generation with example module and CI workflow (13a29dc)nn### Changedn- chore: update version numbers to 0.2.5 in documentation and code (9408e57)n

## [v0.2.5] - 2026-07-14nn### Addedn- feat: add changelog generation to release script (0653753)n- feat: enhance project scaffold generation with example module and CI workflow (13a29dc)n

## [0.1.4] - 2026-07-13

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
- NestJS feature parity: security middleware (CORS, rate limiting, CSRF, security headers)
- NestJS feature parity: validation pipes (`ParseIntPipe`, `ParseFloatPipe`, `ParseBoolPipe`, `ValidationPipe`)
- NestJS feature parity: exception filters with route metadata access and scope precedence
- NestJS feature parity: API versioning (URI prefix, header, media type strategies)
- NestJS feature parity: response serialization with `#[exclude]` / `#[expose(role)]` field-level rules
- NestJS feature parity: compression middleware (gzip, brotli, deflate) via `tower-http`
- NestJS feature parity: WebSocket gateways with `#[web_socket_gateway]`, `#[subscribe_message]`, rooms, and broadcasting
- NestJS feature parity: microservice transport adapters for Redis, RabbitMQ, Kafka (feature-gated)
- NestJS feature parity: cache interceptor with `#[cache(ttl_secs = N)]` route attribute and `CacheMetadata`
- NestJS feature parity: cron scheduling with `cron_schedule()`, `#[cron]`, `#[interval]`, `#[timeout]` markers
- NestJS feature parity: global modules with `#[global]` attribute and `ModuleRef` runtime DI container access
- NestJS feature parity: optional dependencies via `#[injectable(optional = [Type, ...])]`
- NestJS feature parity: custom decorator support with `create_param_decorator!` macro
- New feature flags: `security`, `security-cors`, `security-rate-limit`, `security-headers`, `security-csrf`, `compression`, `versioning`, `serialization`, `validation`, `cron`, `custom-decorators`, `transport-redis`, `transport-rabbitmq`, `transport-kafka`

### Changed

- Renamed project from "RustFrame" to "Ironic"
- Internal `rustframe_*` crate aliases renamed to `ironic_*`
- MSRV bumped from 1.85 to 1.97
- Dependency updates: diesel 2.2.12→2.3.11, jsonwebtoken→9 (pinned), time 0.3.45→0.3.47, hickory-proto 0.25.2→0.26.1
- Fixed 6 Rust 1.97 clippy warnings

### Security

- `.cargo/audit.toml` added to ignore unfixable RUSTSEC-2023-0071 (rsa, transitive via oauth2)
- CI supply-chain job runs `cargo audit` and `cargo deny check`
