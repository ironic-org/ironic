---
title: OpenAPI / Swagger
description: Automatic OpenAPI 3.1 spec generation and Swagger UI from your route definitions.
---

# OpenAPI / Swagger

Ironic auto-generates an [OpenAPI 3.1](https://spec.openapis.org/oas/latest.html) JSON specification from your controllers, routes, and types. A Swagger UI is also available for interactive exploration.

---

## Enable

```toml
[dependencies]
ironic = { features = ["openapi"] }
```

## Configure

Use the builder pattern on `OpenApiConfig`:

```rust
use ironic::openapi::{OpenApiConfig, SecurityScheme};

.platform(
    AxumAdapter::new()
        .with_openapi(
            OpenApiConfig::new("My API", "0.1.0")
                .description("REST API for my application")
                .json_path("/docs/openapi.json")       // default: /openapi.json
                .security_scheme(
                    "bearer",
                    SecurityScheme::HttpBearer {
                        bearer_format: Some("JWT".into()),
                    },
                ),
        )
        .swagger_ui("/docs"),                          // Swagger UI at /docs
)
```

## API Reference

### `OpenApiConfig::new(title, version)`

Creates a new config with the required title and version.

### `.description(description)`

Sets the API description shown in the spec.

### `.json_path(path)`

Changes where the JSON spec is served. Default: `/openapi.json`.

### `.security_scheme(name, scheme)`

Registers a reusable security scheme for the `#[api(security = "...")]` attribute in routes.

| `SecurityScheme` variant | Use |
|--------------------------|-----|
| `SecurityScheme::ApiKey { name, location }` | API key in header/query/cookie |
| `SecurityScheme::HttpBearer { bearer_format }` | HTTP Bearer (JWT) |
| `SecurityScheme::OAuth2AuthorizationCode { authorization_url, token_url, scopes }` | OAuth2 auth code flow |

### `.schema::<T>(name)`

Registers a reusable component schema from a type that implements `OpenApiSchema`.

## Route Annotations

Each route can be documented with attributes:

```rust
#[get("/:id")]
#[api(
    summary = "Get user by ID",
    tag = "Users",
    security = "bearer",          // references the scheme name from config
    deprecated = true,
    operation_id = "getUser",
)]
#[resp(200, "User found", json = User)]
#[resp(404, "User not found")]
async fn get(&self, #[param] id: u64) -> Result<Json<User>, HttpError> {
    // ...
}
```

### `#[api(...)]` attributes

| Attribute | Description |
|-----------|-------------|
| `summary` | Short description |
| `description` | Long description |
| `tag` | Grouping tag |
| `security` | Security scheme name |
| `deprecated` | Mark as deprecated |
| `operation_id` | Unique operation name |

### `#[resp(status, description, json = Type)]`

| Parameter | Description |
|-----------|-------------|
| `status` | HTTP status code |
| `description` | Response description |
| `json = Type` | Response body type (must implement `OpenApiSchema`) |

## Parameter Annotations

Route parameters are automatically documented:

```rust
async fn get(
    #[param] id: u64,              // path parameter
    #[query] filter: Option<String>, // query parameter
    #[header] authorization: String, // header parameter
    #[body] dto: CreateUserDto,    // request body
) -> Result<Json<User>, HttpError>
```

## Generated Files

### JSON Spec

Served at the configured `json_path` (default: `GET /openapi.json`):

```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "My API",
    "version": "0.1.0",
    "description": "REST API for my application"
  },
  "paths": { ... },
  "components": {
    "schemas": { ... },
    "securitySchemes": {
      "bearer": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    }
  }
}
```

### Generate Spec File

```bash
# CLI (builds, starts, fetches, saves, shuts down)
ironic openapi
ironic openapi -p api-gateway -o docs/openapi.json

# Manual
curl http://localhost:8080/openapi.json > spec.json
```

### Generate Client SDK

```bash
npx openapi-typescript spec.json -o client.ts     # TypeScript
openapi-python-client generate --path spec.json    # Python
openapi-generator-cli generate -i spec.json -g go  # Go
```

## What Gets Documented

| Feature | Included |
|---------|----------|
| Routes | All `#[controller]` + `#[routes]` methods |
| Path params | `#[param]` decorators on handler args |
| Query params | `#[query]` decorators on handler args |
| Header params | `#[header]` decorators on handler args |
| Request bodies | `#[body]` decorator + `#[api]` body annotation |
| Response schemas | `#[resp]` attribute + return type |
| Error responses | Standard `HttpError` envelope schema |
| Security schemes | Registered via `.security_scheme()` |
| Tags | Grouped by `#[api(tag = "...")]` |
