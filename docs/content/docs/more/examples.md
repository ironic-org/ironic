---
title: Examples
description: Hello world, REST, validation, errors, versioning, serialization, and isolated application testing.
---

# Examples

- [`examples/hello-world`](../../../examples/hello-world/src/main.rs) demonstrates macros, DI, one
  route, Axum construction, and in-process parity testing.
- [`examples/rest-api`](../../../examples/rest-api/src/main.rs) demonstrates GET and POST routes,
  JSON extraction, validation errors, domain not-found errors, `HealthModule`, OpenAPI schema
  generation, Swagger UI, and `TestApplication`.
- [`examples/versioning`](../../../examples/versioning/src/main.rs) demonstrates URI prefix, header,
  and media-type API versioning strategies.
- [`examples/serialization`](../../../examples/serialization/src/main.rs) demonstrates
  `#[derive(Serializable)]` with `#[exclude]` and field-role based exposure.
- [Validation pipes tests](../../../crates/ironic-pipes/tests/) demonstrate `ValidationPipe` with
  `garde` integration, including parameter-level and body-level validation.
- [Exception filters tests](../../../crates/ironic-exception-filters/tests/) demonstrate
  `ExceptionFilter` chaining at the route, controller, and global level.
- [Ironic testing integration tests](../../../crates/ironic-testing/tests/testing.rs)
  demonstrate provider/value/factory overrides, query/header/body extraction, JSON assertions,
  structured error assertions, and lifecycle cleanup.
- [Explicit API tests](../../../crates/ironic-platform-axum/src/lib.rs) show the same controller
  and route behavior without procedural macros.

Run all examples with:

```bash
cargo test --workspace
```
