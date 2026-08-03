---
title: v1.2.x
description: Complete changelog and release notes for the Ironic v1.2.x stable series.
---

# v1.2.x — Current Stable Series

All versions in the v1.2.x series. Full details in [CHANGELOG.md](https://github.com/ironic-org/ironic/blob/main/CHANGELOG.md).

---

## v1.2.9 — 2026-08-03

### Added
- Transactional outbox and inbox for at-least-once event delivery
- outbox and inbox marker attributes for handler discovery
- Generate sitemap.xml at docs build time (robots.txt already referenced it)
- Docs: 1200x630 OG social card, per-page og:image meta, lazy-route code splitting (14.6MB MDX chunk loaded on navigation only)
### Changed
- Remove version-based blog posts; release.sh now updates releases pages from CHANGELOG.md
- Docs: merge getting-started intro pages, add blog search + related-docs links, clarify section boundaries
- Docs: split testing section into 4 pages, expand provider-health guide, add v1.0.x/v1.2.x migration guides
- Docs: split the 2052-line quick-learn feature-reference into api-reference, architecture, features, and patterns pages
- Docs: expand examples guide with a full blog-api walkthrough and add a numbered 12-step learning path to the getting-started landing page
- Rename #[event_handler] attribute macro to #[event] and #[message_handler] to #[message]
- Docs: split transport events guide into a transport reference and a 3-microservice events tutorial
- Docs: consolidate blog-api into examples, remove duplicated project-structure page, rename message-handler page to message
- Docs: split the 1110-line database-integrations guide into a setup guide plus per-ORM pages (sqlx, seaorm, diesel, mongodb, redis, testing)
- Remove the no-op #[sse] marker attribute; SSE remains programmatic via sse_endpoint and sse_route
### Fixed
- Docs blog pages now render each post's real date and computed read time instead of hardcoded values
- Bump CURRENT_VERSION constant to 1.2.8 and fix release.sh to always sync it on release
- Docs: correct examples path, replace fabricated benchmark numbers with measured cargo bench results
- Fix unbalanced code fences in two docs pages and stale footer links (websocket-gateways, core/fundamentals)
- Docs: remove fabricated chat and e-commerce demos from demo-apps page (only the blog API example exists)

---
## v1.2.8 — 2026-08-01

### Added
- add fetch_latest and poll_interval_ms to KafkaClientConfig and TransportConfig (6a91e13)

### Fixed
- improve TCP socket binding error handling in AxumApplication (01f2df3)

---
## v1.2.7 — 2026-07-28

### Added
- Auto-validation pipe ;  macro now automatically validates , ,  via

---
## v1.2.6 — 2026-07-28

### Changed
- #[event_handler] auto_register is now default — use manual_register to opt out

---
## v1.2.5 — 2026-07-28

### Fixed
- clippy warnings (borrowed_box, collapsible_if, map_unwrap_or, used_underscore_binding) (ec16ebc)

---
## v1.2.4 — 2026-07-28

### Added
- EventClient injection in #[event_handler(transport, auto_register)] — handlers can receive Arc<EventClient> as second param to emit events

---
## v1.2.3 — 2026-07-28

### Added
- Transport-agnostic EventClient/EventServer DI providers with auto-connect/listen lifecycle, TransportConfig/TransportKind config, and transport auto_register support in #[event_handler]
- Integration tests for EventClient/EventServer paired flow, DI resolution, and #[event_handler(transport)] macro
- Documentation page for transport events with beginner-friendly examples
### Fixed
- Generated paths in #[event_handler(transport)] now use microservices:: prefix for MicroserviceServer/EventHandler/TransportError

---
## v1.2.2 — 2026-07-27

### Added
- add support for GraphQL and gRPC resource generation in resource module (0a2300e)
- add ensure_serde_dep function to automatically include serde in Cargo.toml (149f8d0)

### Changed
- remove manual RequestLogging middleware from GraphQL and HTTP app builders docs: update RequestLogging documentation for clarity and custom logger example (7a53fb0)
- update OpenAPI documentation for clarity and structure (3f77cf2)

---
## v1.2.1 — 2026-07-27

### Added
- add serde dependency for serialization in GraphQL, gRPC, HTTP, and monorepo configurations (3a627d3)

---
## v1.2.0 — 2026-07-27

### Added
- add serde dependency for serialization and re-export in the library (7bc0ff9)

### Changed
- simplify dependency extraction and formatting in project and monorepo generators (aeb7918)

---
