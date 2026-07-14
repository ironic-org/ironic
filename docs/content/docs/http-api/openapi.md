---
title: OpenAPI
description: Auto-generate OpenAPI schemas and Swagger UI from your controller definitions — no extra annotations needed.
---

# OpenAPI

## What you'll learn

- Generate OpenAPI specs automatically from your controllers
- Serve Swagger UI at `/docs`
- Add descriptions and examples to your API schema

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

## Try it yourself

1. Create a `ProductView` struct with `id`, `name`, `price`
2. Add `#[derive(OpenApiSchema)]`
3. Run the server and visit `/docs`
4. Verify your Product schema appears in Swagger UI

## What you learned

- [x] `#[derive(OpenApiSchema)]` auto-generates API documentation
- [x] Swagger UI is available at `/docs`
- [x] All controllers, routes, and schemas are discovered automatically
