---
title: OpenAPI Transport
description: Automatic OpenAPI/Swagger documentation generation from route definitions.
---

# OpenAPI Transport

Ironic can automatically generate OpenAPI 3.0 specifications from your controller and route definitions. This includes request/response schemas, path parameters, query parameters, and security schemes.

## Enabling OpenAPI

Enable the `openapi` feature:

```toml
[dependencies]
ironic = { version = "1.0", features = ["openapi"] }
```

## How It Works

OpenAPI documentation is built at compile time by the `#[controller]` and `#[routes]` macros. Each route's metadata — method, path, parameters, request body, response type, and status codes — is collected into an `OpenApiDocument`.

```rust
use ironic::openapi::OpenApiConfig;

let config = OpenApiConfig {
    title: "My API".into(),
    version: "1.0.0".into(),
    description: Some("API for my application".into()),
};

// Automatically served at GET /docs/swagger
```

## Generated Documentation Includes

- **Paths**: All registered routes with methods and parameters
- **Schemas**: Request/response JSON schemas derived from Rust types
- **Security**: OAuth2, JWT bearer, API key schemes
- **Tags**: Grouped by controller name
- **Error Responses**: Standard error envelope schema

## Swagger UI

When enabled, a Swagger UI is served at the `/docs/swagger` endpoint, allowing interactive API exploration.

## Customization

```rust
let config = OpenApiConfig {
    title: "My API".into(),
    version: env!("CARGO_PKG_VERSION").into(),
    servers: vec!["https://api.example.com".into()],
    security: vec![SecurityScheme::BearerJWT],
    // ...
};
```

## Roadmap

- **OpenAPI 3.1 support** (JSON Schema 2020-12)
- **Example generation** from test fixtures
- **Client SDK generation** (via openapi-generator integration)
- **API changelog** from OpenAPI diff between versions
