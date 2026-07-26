---
title: v1.1.x
description: Complete changelog and release notes for the Ironic v1.1.x stable series.
---

# v1.1.x — Current Stable Series

All versions in the v1.1.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v1.1.9 — 2026-07-27

# v1.1.9

### Added
- update dependencies and enhance GraphQL integration in project generator (579a213)
- add email and file upload modules with various backends (b6448bf)
- enhance app generation with gRPC support and modular structure (7ff6139)
- add gRPC support for app generation with `ironic generate app --grpc` (9966126)
### Fixed
- update feature reference documentation to clarify dependencies for various features (fa1456a)
- add dotenvy dependency for environment variable management (49bd20b)
- update dependencies for gRPC support in generated app's Cargo.toml (47f85ca)
### Changed
- Add meta.json for Quick Learn documentation and re-export additional dependencies in lib.rs (4b2ce06)
- release v1.1.8 (fe32090)
- Refactor generator modules to use common utilities (b8b10d6)

---
## v1.1.8 — 2026-07-26

# v1.1.8
### Added
- ironic generate app --grpc for gRPC microservice scaffold

---
## v1.1.7 — 2026-07-26

# v1.1.7

### Added
- enhance app generation with platform module and logging configuration (d46a656)

---
## v1.1.6 — 2026-07-26

# v1.1.6
### Changed
- Simplify generated app template to minimal skeleton
- Split generators/mod.rs into app.rs resource.rs monorepo.rs graphql.rs
### Added
- Generate PRODUCTION.md with production readiness guide for new apps
- ironic openapi command to auto-generate OpenAPI JSON spec
- [profile.release] section to all generated Cargo.toml templates

---
## v1.1.5 — 2026-07-25

# v1.1.5
### Changed
- generate_app creates revallens-style microservice with platform/welcome/example module, per-app Dockerfile/.env, unique port
- app_module uses #[derive(Module)] macro pattern instead of impl Module for
- Remove shared-config and observability libs from monorepo scaffold
- Remove Dockerfile from generated app template
### Added
- ironic dev -p <name> monorepo support with hot reload

---
## v1.1.4 — 2026-07-25

# v1.1.4
### Changed
- New app scaffolding matches RivalLens structure: multi-line #[module()], repositories/tests in module template
- generate controller/service/repository auto-inject into #[module()] providers/controllers arrays

---
## v1.1.3 — 2026-07-25

# v1.1.3

### Added
- enhance app generation with health module and controller scaffolding (aae2a7b)
- auto-convert single-service to monorepo when running ironic generate app (52b2dec)
- monorepo workspace as default project structure — ironic new creates apps/ + libs/ layout (aae39c6)
- add package argument to CargoArgs for monorepo support and update documentation (3aaed0b)
### Changed
- cargo fmt (14acd27)
- update project creation instructions in install script (38e0e66)
- fix clippy -D warnings — add .. patterns for CargoArgs, change execute to take reference (bdecd4c)
- remove outdated Production Release Guide (0a63098)
- Refactor lifecycle documentation and add new hooks (ae10eed)
- Enhance documentation for Container Override, Dependency Management, Service Lifetimes, and Hot-Reload Config (302fa22)
- enhance circular dependencies documentation with detailed explanations and examples (2299cd0)
- simplify module definitions using derive macros for improved readability (a5b2a19)
- update module definitions to use derive macros for cleaner syntax (8a32a48)
- add detailed explanation of monorepo architecture and service interactions (0d617f4)
- Add project structure documentation and new modules (a17024d)
- remove microservices example and related proto files (231267c)
- fix clippy issues in app_module generator (b40649e)
- fix unused variable warning in app_module (9b98b09)

---
## v1.1.2 — 2026-07-25

# v1.1.2

### Added
- add package argument to CargoArgs for monorepo support and update documentation (3aaed0b)
### Changed
- remove outdated Production Release Guide (0a63098)
- Refactor lifecycle documentation and add new hooks (ae10eed)
- Enhance documentation for Container Override, Dependency Management, Service Lifetimes, and Hot-Reload Config (302fa22)
- enhance circular dependencies documentation with detailed explanations and examples (2299cd0)
- simplify module definitions using derive macros for improved readability (a5b2a19)
- update module definitions to use derive macros for cleaner syntax (8a32a48)
- add detailed explanation of monorepo architecture and service interactions (0d617f4)
- Add project structure documentation and new modules (a17024d)
- remove microservices example and related proto files (231267c)
- fix clippy issues in app_module generator (b40649e)
- fix unused variable warning in app_module (9b98b09)

---
## v1.1.1 — 2026-07-25

# v1.1.1

### Added
- implement MQTT and NATS live transports, Federation helper, MS pipeline docs (80808bb)
- add GraphQL integration with resolver scaffolding and proc-macro support (6cd0219)
- Add support for hybrid applications and microservice transport (71caafb)
- Add new specifications for CLI enhancements, cookie utilities, and microservice architecture (9d7a59c)
- update CURRENT_VERSION to 1.1.0 (ced4bb3)
- release v1.1.0 with new features, fixes, and documentation updates (e09f8f2)
- Add comprehensive documentation and tests for various components (ed93dc2)
- add Server-Sent Events (SSE) support with SseRoute and SseConfig (6d408e1)
- add #[mcp_tool] macro for generating MCP tools with JSON Schema inference (0c3863f)
- implement MCP transport with JSON-RPC 2.0 support and documentation (2eca060)
- add DocsThemeCustomizer component to AppRoot for enhanced theming (b08f592)
- implement GitHub stats display in sidebar and clean up layout components (f6b418c)
- enhance documentation and testing across all crates (6afa3e5)
- update documentation and components for GitHub integration, including version bump and badge redesign (bccf94e)
- enhance changelog management with new scripts for entry addition and release processing (f01e1d9)
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
- add missing entries to .gitignore for openspace, opencode, and claude (3f82210)
- update navigation and introduction links for consistency (2bca747)
- remove unnecessary --offline flag from cargo test command (8cb4770)
- simplify titles in getting started and project structure documentation (d513d62)
- update examples section for clarity and consistency (fbb74b6)
- replace newline character syntax for better compatibility (73aa191)
- add winnow duplicate to cargo-deny skip list (bd85f3f)
- update response body mapping in platform adapter documentation (e63a720)
- handle missing environment variables in BuildInfo (39e64f1)
- update CURRENT_VERSION to 1.0.3 (42468f6)
### Changed
- update links and add new content for improved navigation and clarity (14234a6)
- comprehensive NestJS vs Ironic feature map covering all 132 features (0b094a8)
- update coverage map to 89% — 118/132 items complete (30fafb3)
- improve code quality and readability across multiple modules (719314d)
- resolve all warnings, clippy issues, and formatting for release (5c07039)
- resolve all warnings and clippy issues, final cleanup (044e914)
- update NestJS vs Ironic coverage map to reflect 76% parity (6b01212)
- Remove deprecated OpenSpec commands and skills (a0570ba)
- release v1.0.9 (31f5cd7)
- update documentation for API changes and improve clarity in examples (b9c40d3)
- Enhance documentation and update changelog for versions v0.4.3 to v1.0.9 (8eb0cff)
- Update release notes and documentation for v0.4.x and v1.0.x series (1e4b694)
- Remove deprecated lifecycle hooks and related documentation (47b8cd8)
- Add lifecycle hooks documentation and transport features (ccb19b3)
- update changelog for v0.4.x and v1.0.x releases (e7d81f3)
- release v1.0.9 (4596235)
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
## v1.1.0 — 2026-07-23

# v1.1.0
### Added
- release.sh: prefer [Unreleased] content over git log when non-empty
- add-changelog-entry.sh: helper script for quick [Unreleased] entries
- Created transport documentation group with HTTP, WebSocket, GraphQL, OpenAPI, and MCP pages
- Lifecycle hooks section with 15 detailed hook pages as main sidebar nav
- Configuration section with env cascade, alternative sources, from_env, and env-var reference pages
- MCP transport implementation — McpServer, McpRouter, McpTool with JSON-RPC 2.0 over HTTP, AxumAdapter integration, and docs
- #[mcp_tool] proc-macro — infer JSON Schema from Rust function parameters, auto-generate McpTool
- RedisQueue backend with BRPOP/RPUSH, priority queues, retry tracking, TTL expiry, dead-letter support, and QueueConfig
- Completed RedisCache backend with GET/SETEX/DEL/SCAN-based prefix eviction
- #[cache_key] and #[cache_ttl] marker attributes for declarative cache configuration
- #[event_handler] proc-macro that generates EventBus subscriber registration with configurable capacity
- SSE framework integration with SseRoute, SseConfig, SseError, reconnection support, and #[sse] marker attribute
- EventBroadcaster type alias and AxumAdapter::sse_route() for broadcast-based SSE endpoints
### Fixed
- release.sh: macOS compat — replace head -n -1 with sed '$d'
- add-changelog-entry.sh: handle pipefail grep exits with || true
- add-changelog-entry.sh: prevent duplicate category insertion
- Changelog entries use real newlines instead of literal backslash-n in markdown output
- Critical API doc mismatches — health paths, HealthRegistry/PasswordHasher/MetricsRegistry constructors, inject_trace_context, metric signatures, key_resolver name
### Changed
- Added comprehensive doc comments and test modules across all 22 crates
- Fixed 12 failing unit tests and 4 broken doctests
- Consolidated docs: removed core/hooks/ (duplicate of lifecycle/), deduplicated caching/scheduling/websocket pages, added 3 orphaned getting-started pages to sidebar
- Standardized changelog format across all release docs — consistent bullet points, dates, deduplicated headings, fixed n artifacts
### Changed
- Docs: redesigned GitHub star/fork badges with polished inline pill design
- Docs: consolidated duplicated GitHubStarsBadge into shared component
- Docs: added live GitHub stars and forks to StatsBar and Footer
- Docs: extracted GITHUB_OWNER/GITHUB_REPO/GITHUB_URL to constants

---
