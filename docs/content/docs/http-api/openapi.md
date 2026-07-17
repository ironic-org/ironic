---
title: OpenAPI
description: Auto-generate OpenAPI 3.1 specs and Swagger UI from route attributes — no extra boilerplate.
---

# OpenAPI

## What you'll learn

- Annotate handlers with `#[api]`, `#[resp]`, `#[body]` for full OpenAPI metadata
- Serve Swagger UI at `/docs`
- Document request body, responses per status code, and parameters
- Configure security schemes (Bearer JWT, API key, OAuth2)

---

## How it works

OpenAPI docs build from:

1. **`#[derive(OpenApiSchema)]`** on your DTOs/entities → JSON Schema
2. **`#[api]`, `#[resp]`, `#[body]`** on route handlers → operation metadata

No separate route definition needed — everything stays on the handler method.

---

## Step 1: Schema derive

```rust
use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, OpenApiSchema)]
struct UserView {
    /// Unique identifier.
    id: u64,
    /// Display name.
    name: String,
}

#[derive(Deserialize, OpenApiSchema)]
struct CreateUser {
    /// Full name (1–100 characters).
    name: String,
    /// Age in years.
    age: Option<u16>,
}
```

Doc comments (`///`) become `description` in the schema.

---

## Step 2: Annotate handlers

```rust
#[routes]
impl ExampleController {
    #[get]
    #[api(summary = "List all examples", tag = "Examples")]
    #[resp(200, "A list of examples", json = Vec<Example>)]
    async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {
        Ok(Json(self.service.list()))
    }

    #[get("/:id")]
    #[api(summary = "Get an example by ID", tag = "Examples")]
    #[resp(200, "The requested example", json = Example)]
    #[resp(404, "Example not found")]
    async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {
        self.service.find(id).map(Json)
    }

    #[post]
    #[api(summary = "Create a new example", tag = "Examples")]
    #[body(json = CreateExampleDto)]
    #[resp(201, "Example created", json = Example)]
    #[resp(400, "Validation error")]
    async fn create(&self, #[body] dto: CreateExampleDto) -> Result<Json<Example>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }

    #[put("/:id")]
    #[api(summary = "Update an existing example", tag = "Examples")]
    #[body(json = UpdateExampleDto)]
    #[resp(200, "Example updated", json = Example)]
    #[resp(404, "Example not found")]
    async fn update(&self, #[param] id: u64, #[body] dto: UpdateExampleDto) -> Result<Json<Example>, HttpError> {
        self.service.update(id, dto).map(Json)
    }

    #[delete("/:id")]
    #[api(summary = "Delete an example", tag = "Examples")]
    #[resp(204, "Example deleted")]
    #[resp(404, "Example not found")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {
        self.service.delete(id)
    }
}
```

The `#[routes]` macro reads `#[api]`, `#[resp]`, `#[body]` and generates the `.openapi(...)` call automatically. Your handler methods stay clean.

---

## Attribute reference

### `#[api(key = "value", ...)]`

| Key | Type | Description |
|-----|------|-------------|
| `summary` | string | Short route description |
| `tag` | string | Grouping tag (repeatable) |
| `operation_id` | string | Unique operation ID |
| `security` | string | Security scheme name (repeatable, registered in `OpenApiConfig`) |

```rust
#[api(summary = "Get user by ID", tag = "Users", operation_id = "getUser", security = "bearer")]
```

### `#[resp(status, "description", json = Type)]`

First two args are required; `json = Type` is optional (for responses with a body).

```rust
#[resp(200, "User found", json = UserView)]
#[resp(404, "User not found")]
```

### `#[body(json = Type)]`

```rust
#[body(json = CreateUser)]
```

---

## Step 3: Wire it up in main.rs

```rust
use ironic::{AxumAdapter, OpenApiConfig, OpenApiAxumExt};

AxumAdapter::new()
    .with_openapi(
        OpenApiConfig::new("My API", "0.1.0")
            .description("REST API for my app")
            .security_scheme("bearer", SecurityScheme::HttpBearer {
                bearer_format: Some("JWT".into()),
            }),
    )
    .swagger_ui("/docs")
```

---

## Security schemes

### Available scheme types

```rust
use std::collections::BTreeMap;

// Bearer JWT
SecurityScheme::HttpBearer { bearer_format: Some("JWT".into()) }

// API key in header
SecurityScheme::ApiKey { name: "X-API-Key".into(), location: "header".into() }

// OAuth2 authorization code flow
SecurityScheme::OAuth2AuthorizationCode {
    authorization_url: "https://auth.example.com/authorize".into(),
    token_url: "https://auth.example.com/token".into(),
    scopes: BTreeMap::from([
        ("users:read".into(), "Read user profiles".into()),
    ]),
}
```

### Registering multiple schemes

Chain `.security_scheme(...)` for each scheme — all appear in the `components/securitySchemes` section of the spec:

```rust
use std::collections::BTreeMap;
use ironic::SecurityScheme;

let config = OpenApiConfig::new("My API", "1.0.0")
    .security_scheme("bearer", SecurityScheme::HttpBearer {
        bearer_format: Some("JWT".into()),
    })
    .security_scheme("api_key", SecurityScheme::ApiKey {
        name: "X-API-Key".into(),
        location: "header".into(),
    })
    .security_scheme("oauth", SecurityScheme::OAuth2AuthorizationCode {
        authorization_url: "https://auth.example.com/authorize".into(),
        token_url: "https://auth.example.com/token".into(),
        scopes: BTreeMap::from([
            ("users:read".into(), "Read user profiles".into()),
        ]),
    });
```

### Applying schemes to routes

Use `security = "scheme_name"` in `#[api(...)]` to require a scheme on a specific route. Repeat for multiple schemes:

```rust
#[api(
    summary = "List all examples",
    tag = "Examples",
    security = "bearer",
    security = "api_key",
)]
#[resp(200, "A list of examples", json = Vec<Example>)]
async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {
    Ok(Json(self.service.list()))
}
```

Routes without `security = "..."` are unauthenticated (public).

### Choosing between schemes

| Scenario | Approach |
|----------|----------|
| Single auth method (e.g. JWT only) | One `security_scheme("bearer", ...)` + `security = "bearer"` on protected routes |
| Multiple auth methods, same routes | Multiple `security_scheme` calls + `security = "bearer"` + `security = "api_key"` on each protected route |
| Different auth per route | Apply different `security = "..."` values per handler |
| Mixed public + protected | Omit `security = "..."` on public routes |
| Scoped OAuth2 | Pass scopes via the third argument to `.security(name, scopes)` when building programmatically |

---

## The generated template

`ironic new my-app` generates a full CRUD with:

- `src/main.rs` — `with_openapi(...).swagger_ui("/docs")`
- `src/modules/example/` — Handlers annotated with `#[api]`, `#[resp]`, `#[body]`
- DTOs with `#[derive(OpenApiSchema)]`

Visit `http://localhost:8080/docs` after starting the server.

---

## Generated JSON example

```json
{
  "openapi": "3.1.0",
  "info": { "title": "My API", "version": "0.1.0" },
  "paths": {
    "/example": {
      "get": {
        "summary": "List all examples",
        "operationId": "list",
        "tags": ["Examples"],
        "responses": {
          "200": {
            "description": "A list of examples",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/Example" }
                }
              }
            }
          }
        }
      },
      "post": {
        "summary": "Create a new example",
        "operationId": "create",
        "tags": ["Examples"],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/CreateExampleDto" }
            }
          }
        },
        "responses": {
          "201": {
            "description": "Example created",
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/Example" }
              }
            }
          },
          "400": { "description": "Validation error" }
        }
      }
    }
  }
}
```
