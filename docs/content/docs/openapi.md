---
title: OpenAPI and Swagger UI
description: Generate OpenAPI 3.1 JSON from compiled routes and serve Swagger UI with Axum.
---

# OpenAPI and Swagger UI

OpenAPI and the Axum adapter are included in the main crate:

```toml
[dependencies]
ironic = "0.1"
```

Derive JSON Schemas for request and response DTOs, register reusable components, and wrap the
adapter:

```rust
use ironic::{AxumAdapter, OpenApiAxumExt, OpenApiConfig, OpenApiSchema};

#[derive(OpenApiSchema)]
struct CreateItem {
    name: String,
}

let adapter = AxumAdapter::new()
    .with_openapi(
        OpenApiConfig::new("Items API", "1.0.0")
            .description("Item management API")
            .schema::<CreateItem>("CreateItem"),
    )
    .swagger_ui("/docs");
```

The wrapper discovers every compiled framework route and serves OpenAPI 3.1 JSON at
`/openapi.json`. Ironic route parameters such as `/:id` become OpenAPI parameters such as
`/{id}`. The UI path and JSON path are validated before startup and cannot replace an existing GET
route.

## Operation metadata

Explicit routes can attach summaries, tags, stable operation IDs, parameters, request bodies,
responses, examples, and security requirements:

```rust
use ironic::{
    OpenApiOperation, OpenApiRequestBody, OpenApiResponse, OpenApiRouteExt,
};

let route = route.openapi(
    OpenApiOperation::new()
        .summary("Create an item")
        .operation_id("createItem")
        .tag("items")
        .request_body(OpenApiRequestBody::json::<CreateItem>())
        .response("201", OpenApiResponse::new("Created")),
);
```

Register API-key, HTTP bearer, or OAuth 2 authorization-code schemes with
`OpenApiConfig::security_scheme`, then reference the registered name through
`OpenApiOperation::security`.

Swagger UI assets load from jsDelivr in the browser. The generated JSON endpoint remains local and
works without those assets. Production deployments with a restrictive content security policy
should proxy or self-host Swagger UI assets instead of enabling the provided convenience page.
