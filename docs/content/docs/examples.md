---
title: Examples
description: Hello world, REST, validation, errors, and isolated application testing.
---

# Examples

- [`examples/hello-world`](../../../examples/hello-world/src/main.rs) demonstrates macros, DI, one
  route, Axum construction, and in-process parity testing.
- [`examples/rest-api`](../../../examples/rest-api/src/main.rs) demonstrates GET and POST routes,
  JSON extraction, validation errors, domain not-found errors, `HealthModule`, OpenAPI schema
  generation, Swagger UI, and `TestApplication`.
- [Ironic testing integration tests](../../../crates/ironic-testing/tests/testing.rs)
  demonstrate provider/value/factory overrides, query/header/body extraction, JSON assertions,
  structured error assertions, and lifecycle cleanup.
- [Explicit API tests](../../../crates/ironic-platform-axum/src/lib.rs) show the same controller
  and route behavior without procedural macros.

Run all examples with:

```bash
cargo test --workspace
```
