---
title: "OpenAPI 3.1 Schema Generation — from compiled routes to Swagger UI"
description: "How Ironic introspects compiled routes, derives JSON Schema from Rust type information, and serves a complete OpenAPI 3.1 document with an inline Swagger UI — all at compile time."
date: "2026-07-15"
author: "Ironic Team"
---

# OpenAPI 3.1 Schema Generation — from compiled routes to Swagger UI

Most web frameworks treat API documentation as an afterthought — a YAML file you maintain by hand, drifting out of sync with your codebase with every refactor. Ironic inverts this: your route definitions are the source of truth, and the OpenAPI 3.1 document is a derived artifact. No separate files, no drift, no "the docs say this field is required but the code doesn't." Here's the full pipeline.

## `OpenApiDocument::from_application()` — the entry point

The journey starts at `OpenApiDocument::from_application()` (`crates/ironic-openapi/src/document.rs:439`). It receives a `CompiledHttpApplication` — the in-memory representation of every route your application has registered — and an `OpenApiConfig` carrying metadata like the API title, version, description, and optional security schemes.

It iterates over `application.routes()` and, for each route, pulls out the `OpenApiOperation` metadata stored in the route's extension map. This metadata is populated by proc macros you attach to your controller methods. For example:

```rust
#[openapi(
    summary = "List users",
    operation_id = "listUsers",
    tag = "users"
)]
pub async fn index(&self, _ctx: &RequestContext) -> Result<Json<Vec<User>>, HttpError> {
    // ...
}
```

The `#[openapi]` attribute expands at compile time into a builder chain that constructs an `OpenApiOperation` struct with summary, description, operation ID, tags, parameters, request body schema, and response schemas. This struct gets stored in the route's `RouteMetadata` extension map — a type-erased key-value store that the HTTP kernel is completely unaware of, keeping the core framework decoupled from OpenAPI concerns.

## The `OpenApiSchema` trait and its type-level hierarchy

The real magic is in how types become JSON Schema. The `OpenApiSchema` trait (`crates/ironic-openapi/src/schema.rs:9`) has a single associated method:

```rust
pub trait OpenApiSchema {
    fn openapi_schema() -> Value;
}
```

No `&self`, no runtime instance. This is a purely static, type-driven function — it generates the schema from the type alone. The framework implements this trait across an entire hierarchy of Rust types:

**Primitives.** Integer types (`i8` through `u128`, `isize`, `usize`) all map to `{"type": "integer"}`. Floats (`f32`, `f64`) map to `{"type": "number"}`. `bool` becomes `{"type": "boolean"}`. Both `String` and `str` produce `{"type": "string"}`. These are defined via declarative macros that stamp out identical implementations for every numeric type variant.

**Optionals.** `Option<T>` delegates to `T::openapi_schema()` and adds `"nullable": true`. This is the only place where the trait mutates an existing schema value — it clones the inner schema's object map and injects the nullable flag.

**Collections.** `Vec<T>` produces `{"type": "array", "items": T::openapi_schema()}`. Fixed-size arrays `[T; N]` go further: they emit `minItems` and `maxItems` constraints set to `N`, so the generated spec accurately describes a JSON array with an exact element count. `HashMap<String, T, S>` (with any hasher) and `BTreeMap<String, T>` both emit `{"type": "object", "additionalProperties": T::openapi_schema()}`, correctly modeling the map-to-value semantics.

**Nested structs.** This is where the proc macro ecosystem takes over. The `#[derive(OpenApiSchema)]` macro on a struct generates an `impl OpenApiSchema` that walks every field, calls `FieldType::openapi_schema()` for each, and assembles a JSON Schema object with `"type": "object"`, `"properties"`, and `"required"` arrays. Nested structs compose: if `User` has a field `address: Address`, and `Address` also derives `OpenApiSchema`, the generated schema for `User` will inline a `$ref` or nested object definition for the address property. The trait's recursive structure means you get correct, fully-resolved schemas for arbitrarily deep type hierarchies without writing a single line of OpenAPI YAML.

## Handling path parameters, response schemas, and edge cases

Back in `from_application()`, each route's `OpenApiOperation` metadata is converted into the OpenAPI path-item format. The `operation_json()` helper function is where the details come together: documented parameters are serialized, but so are **undocumented path parameters**. It scans the path template for `{name}` segments, and for any path parameter that hasn't been explicitly documented, it generates a default `String` parameter in the `path` location. This ensures your OpenAPI spec is never missing path parameters, even if you haven't annotated every one.

If a handler has no explicit response schemas defined, the generator inserts a default `"200": {"description": "Successful response"}`. This guarantees that every endpoint in the generated document has at least one response status — OpenAPI tools choke on operations with zero responses.

Operation IDs are also checked for uniqueness. Each route contributes an explicit `operation_id` (from the proc macro) or a generated fallback based on the handler name and route index. A `HashSet` tracks every ID; if a duplicate is found, `from_application()` returns `Err(OpenApiError::DuplicateOperationId { operation_id })`. The test `rejects_duplicate_operation_ids_and_generated_endpoint_conflicts` (`openapi.rs:173`) verifies this: it registers two routes with the same explicit `operation_id` and asserts that document generation fails with the expected error variant. This is a compile-time-caught bug disguised as a runtime-check — your CI pipeline catches the conflict the moment your tests run.

## The Axum adapter: injecting `/openapi.json` and `/docs`

The `OpenApiAxumAdapter` (`crates/ironic-openapi/src/axum.rs:27`) wraps the standard `AxumAdapter` and extends it with OpenAPI route injection. When you call:

```rust
AxumAdapter::new()
    .with_openapi(OpenApiConfig::new("My API", "1.0.0"))
    .swagger_ui("/docs")
```

...the build pipeline does three things before the server starts:

1. Validates that `/openapi.json` and `/docs` don't conflict with any of your application routes (`ensure_endpoint_available` at line 95). If a conflict exists, it returns `Err(OpenApiError::EndpointConflict)`, failing fast before the socket binds.

2. Calls `OpenApiDocument::from_application()` to generate the complete OpenAPI 3.1 JSON document.

3. Calls `openapi_router()` to mount two routes on the native Axum `Router`: a `GET /openapi.json` endpoint that serves the serialized document with the correct `Content-Type`, and an optional `GET /docs` endpoint that serves the Swagger UI HTML page.

The JSON document is stored in an `Arc<String>` so that concurrent requests avoid re-serialization — the document is generated once at startup and shared immutably across every incoming OpenAPI request.

## The Swagger UI page: a compile-time HTML constant

The Swagger UI is served by `swagger_html()` (`axum.rs:174`). This function returns a `String` — not a path to a template file, not a handlebars render, but a literal HTML string compiled directly into the binary. The function is pure: it takes a title and the JSON endpoint path, escapes them for HTML safety, and interpolates them into a format string:

```rust
fn swagger_html(title: &str, json_path: &str) -> String {
    let title = escape_html(title);
    let json_path = serde_json::to_string(json_path)
        .expect("serializing a string cannot fail");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — Swagger UI</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/...">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/..."></script>
  <script>SwaggerUIBundle(...);</script>
</body>
</html>"#
    )
}
```

The HTML template itself requires zero filesystem access, zero template engine, and zero allocation beyond the final `String`. The Swagger UI JavaScript and CSS bundles are fetched from a CDN on the client side — a deliberate tradeoff. The server binary stays small and self-contained (no embedded static asset bundles), while the browser loads the full Swagger UI interactivity from a cacheable CDN URL.

What happens at build time is worth emphasizing: the entire OpenAPI pipeline — route introspection, schema generation, duplicate operation ID detection, endpoint conflict checking — runs before the server binds its port. If anything is wrong, you get a Rust `Err` value, not a 500 from a running server. This is the central philosophy: your API contract is verified as early as possible, as close to compilation as the type system allows.

The result is a living OpenAPI document that **cannot** drift from your code — because it **is** your code, introspected and rendered into a spec. Every field you add to a struct, every route parameter you declare, every response type you change — all of it flows into the generated document automatically. No YAML. No drift. Just the truth.
