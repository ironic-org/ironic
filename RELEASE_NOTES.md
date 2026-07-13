# Ironic 0.1 preview release notes

Ironic 0.1 is the first usable preview of the modular Rust application framework described by the project RFCs. It targets Rust 1.85 and Edition 2024.

## Highlights

- Validated modules, dependency injection, lifecycle hooks, and deterministic shutdown.
- Transport-neutral HTTP contracts with an Axum adapter and Tower/Axum escape hatches.
- Controller, route, extraction, guard, interceptor, and application macros backed by public kernel APIs.
- Socket-free integration testing with local provider overrides.
- Deterministic CLI project and resource generators.
- Typed layered configuration, validation, and redacted secrets.
- Request correlation spans, health endpoints, a 1 MiB request-body limit, and a 30-second request timeout by default.
- Optional OpenAPI 3.1 route discovery, derived DTO schemas, authentication metadata, JSON export, and Swagger UI.

## Benchmark snapshot

Measured on Darwin 25.5.0 arm64 with rustc 1.85.0 in release mode. These single-process measurements are a development baseline, not a cross-machine performance guarantee.

| Operation | Time |
| --- | ---: |
| Module graph compilation | 866 ns/op |
| Route registration | 436 ns/op |
| Transient provider resolution | 157 ns/op |
| HTTP runtime startup | 555 ns/op |
| Ironic in-process request | 780 ns/op |
| Equivalent raw Axum request | 319 ns/op |

Reproduce the snapshot with `cargo bench -p ironic --bench overhead`.

## Known limitations

- The release is experimental and APIs may change before stability guarantees are introduced.
- Axum is the only concrete platform adapter.
- Providers support singleton and transient scopes; request scope is deferred.
- Configuration sources are synchronous. Dynamic, lazy, conditional, and async-configured modules are deferred.
- Authentication helpers, persistence integrations, queues, WebSockets, SSE, and GraphQL remain outside the current scope.
- Panic isolation requires an unwind build. A `panic = "abort"` build still terminates the process.
- `OpenApiSchema` currently derives schemas for named, non-generic structs. Macro routes are
  discovered automatically, while detailed per-operation metadata currently uses explicit route
  definitions.
- The benchmark harness is intentionally dependency-free and reports elapsed wall-clock averages; use application-specific load tests for capacity planning.
