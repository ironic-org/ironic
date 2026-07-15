---
title: OpenAPI
description: Auto-generate OpenAPI schemas and Swagger UI from your controller definitions — no extra annotations needed.
---

# OpenAPI

## What you'll learn

- Generate OpenAPI specs automatically from your controllers
- Serve Swagger UI at `/docs`
- Add descriptions and examples to your API schema
- Customize API metadata, server URLs, and security schemes
- Exclude routes from documentation

---

## Step 1: Add the schema derive

```rust
use ironic::OpenApiSchema;
use serde::Serialize;

#[derive(Serialize, OpenApiSchema)]    // ← Auto-generates OpenAPI schema
struct UserView {
    id: u64,
    name: String,
    email: String,
}
```

## Step 2: Build and view

```rust
#[ironic::main]
async fn main() {
    let app = FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build().await.unwrap();

    // Print OpenAPI JSON (useful for CI/CD pipelines)
    println!("{}", ironic::generate_openapi_json(&app));

    app.listen("127.0.0.1:3000").await.unwrap();
}
```

Visit **`http://localhost:3000/docs`** for the Swagger UI:

```
┌─────────────────────────────────────────┐
│  Swagger UI                     [docs]  │
│  ┌─────────────────────────────────────┐│
│  │ GET    /users          List users  ││
│  │ POST   /users          Create user ││
│  │ GET    /users/:id      Get user    ││
│  │ PUT    /users/:id      Update user ││
│  │ DELETE /users/:id      Delete user ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

> **It just works.** Every controller and route is automatically discovered. No extra code needed.

## Customizing schema fields

Add descriptions and examples to your DTO fields using doc comments. The OpenAPI generator picks them up automatically:

```rust
#[derive(Serialize, OpenApiSchema)]
struct ProductView {
    /// Unique product identifier (auto-incremented)
    id: u64,

    /// Human-readable product name shown in listings
    name: String,

    /// Price in cents (e.g. 1999 = $19.99)
    #[serde(rename = "price_cents")]
    price: u64,
}
```

Doc comments (`///`) become field-level `description` properties. `#[serde(rename)]` changes the field name in the schema — useful when your internal Rust names differ from your public API.

## Setting API info

Customize the OpenAPI `info` block via `OpenApiConfig`:

```rust
use ironic::openapi::OpenApiConfig;

let config = OpenApiConfig {
    title: "PetStore API".into(),
    version: "2.1.0".into(),
    description: Some("REST API for managing pets, orders, and inventory.".into()),
    ..Default::default()
};

FrameworkApplication::builder()
    .platform(AxumAdapter::new())
    .openapi_config(config)
    .build().await.unwrap();
```

This produces:

```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "PetStore API",
    "version": "2.1.0",
    "description": "REST API for managing pets, orders, and inventory."
  }
}
```

## Server URLs

If your API runs behind a gateway or load balancer, set the public-facing URL:

```rust
let config = OpenApiConfig {
    servers: vec![
        "https://api.example.com/v2".into(),
        "https://staging-api.example.com/v2".into(),
    ],
    ..Default::default()
};
```

The Swagger UI uses the first server URL for "Try it out" requests.

## Security schemes

Document authentication so clients know what to send:

```rust
let config = OpenApiConfig {
    security: vec![SecurityScheme::BearerJwt {
        description: "Include `Authorization: Bearer <token>`".into(),
    }],
    ..Default::default()
};
```

| Scheme | Use case |
|--------|----------|
| `SecurityScheme::BearerJwt` | Token-based auth; adds a padlock icon to every route in Swagger UI |
| `SecurityScheme::ApiKey { header }` | API key in a custom header like `X-API-Key` |
| `SecurityScheme::Basic` | HTTP Basic Auth (username/password) |

Security schemes apply globally. To mark specific routes as public (no auth), use `#[no_auth]` on the route handler.

## Tag grouping for controllers

Organize Swagger UI into logical sections with tags:

```rust
#[controller("/products")]
#[openapi_tag("Inventory")]
#[derive(Injectable)]
struct ProductsController {
    service: Arc<ProductsService>,
}
```

All routes in `ProductsController` appear under the "Inventory" group in the Swagger UI sidebar, separate from "Users" or "Orders" controllers.

## Enabling Swagger UI

Swagger UI is served at `/docs` automatically when the `openapi` feature is active. No additional feature flag is needed:

```toml
ironic = { features = ["openapi"] }
```

The JSON spec is available at `/docs/openapi.json`. Use this endpoint for CI/CD tooling, API gateways, or generating client SDKs.

## Excluding routes

Hide internal or deprecated routes from the docs:

```rust
#[get("/health")]
#[hidden]                              // ← This route won't appear in OpenAPI
async fn health(&self) -> StatusCode {
    StatusCode::OK
}
```

Use `#[hidden]` for health checks, metrics endpoints, debug routes, or any endpoint you don't want publicly documented.

## Generated JSON example

Here's what a minimal OpenAPI output looks like for a `GET /users` endpoint:

```json
{
  "openapi": "3.1.0",
  "info": { "title": "My API", "version": "0.1.0" },
  "paths": {
    "/users": {
      "get": {
        "summary": "List users",
        "operationId": "listUsers",
        "responses": {
          "200": {
            "description": "Successful response",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/UserView" }
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "UserView": {
        "type": "object",
        "properties": {
          "id": { "type": "integer", "format": "int64" },
          "name": { "type": "string" },
          "email": { "type": "string" }
        }
      }
    }
  }
}
```

## Common mistakes

| Mistake | Why it's wrong | Fix |
|---------|---------------|-----|
| Forgetting `#[derive(OpenApiSchema)]` | DTO won't appear in `components/schemas` | Add the derive to every struct used in routes |
| Leaking internal routes in docs | Health checks and debug endpoints clutter Swagger UI | Use `#[hidden]` on routes meant only for internal use |
| Not setting `info.version` | API consumers can't track breaking changes | Always set a version string in `OpenApiConfig` |
| Omitting security schemes | Security requirements are invisible to API consumers | Add `SecurityScheme::BearerJwt` or `ApiKey` to `OpenApiConfig` |

## Try it yourself

1. Create a `ProductView` struct with `id`, `name`, `price`
2. Add `#[derive(OpenApiSchema)]`
3. Run the server and visit `/docs`
4. Verify your Product schema appears in Swagger UI

## What you learned

- [x] `#[derive(OpenApiSchema)]` auto-generates API documentation
- [x] Swagger UI is available at `/docs`
- [x] All controllers, routes, and schemas are discovered automatically
- [x] Doc comments (///) become field descriptions in the schema
- [x] `OpenApiConfig` sets title, version, description, and server URLs
- [x] Security schemes document authentication requirements
- [x] `#[hidden]` excludes routes from the OpenAPI spec
- [x] `#[openapi_tag]` organizes endpoints into Swagger UI groups
