---
title: v1.1.x
description: Complete changelog and release notes for the Ironic v1.1.x stable series.
---

# v1.1.x — Current Stable Series

All versions in the v1.1.x series. Visit the [Blog](/blog) for detailed release announcements.

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
