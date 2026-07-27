---
title: v1.2.x
description: Complete changelog and release notes for the Ironic v1.2.x stable series.
---

# v1.2.x — Current Stable Series

All versions in the v1.2.x series. Visit the [Blog](/blog) for detailed release announcements.

---

## v1.2.4 — 2026-07-28

# v1.2.4
### Added
- EventClient injection in #[event_handler(transport, auto_register)] — handlers can receive Arc<EventClient> as second param to emit events

---
## v1.2.3 — 2026-07-28

# v1.2.3
### Added
- Transport-agnostic EventClient/EventServer DI providers with auto-connect/listen lifecycle, TransportConfig/TransportKind config, and transport auto_register support in #[event_handler]
- Integration tests for EventClient/EventServer paired flow, DI resolution, and #[event_handler(transport)] macro
- Documentation page for transport events with beginner-friendly examples
### Fixed
- Generated paths in #[event_handler(transport)] now use microservices:: prefix for MicroserviceServer/EventHandler/TransportError

---
## v1.2.2 — 2026-07-27

# v1.2.2

### Added
- add support for GraphQL and gRPC resource generation in resource module (0a2300e)
- add ensure_serde_dep function to automatically include serde in Cargo.toml (149f8d0)
### Changed
- remove manual RequestLogging middleware from GraphQL and HTTP app builders docs: update RequestLogging documentation for clarity and custom logger example (7a53fb0)
- update OpenAPI documentation for clarity and structure (3f77cf2)

---
## v1.2.1 — 2026-07-27

# v1.2.1

### Added
- add serde dependency for serialization in GraphQL, gRPC, HTTP, and monorepo configurations (3a627d3)

---
## v1.2.0 — 2026-07-27

# v1.2.0

### Added
- add serde dependency for serialization and re-export in the library (7bc0ff9)
### Changed
- simplify dependency extraction and formatting in project and monorepo generators (aeb7918)

---
