---
title: Examples
description: Real-world example applications built with Ironic — REST APIs, WebSockets, validation, error handling, and testing.
---

# Examples

Each example is a complete, runnable project:

| Example | What it demonstrates |
|---------|---------------------|
| [hello-world](https://github.com/ironic-org/ironic/tree/main/examples/hello-world) | Minimal API with a controller, service, and JSON responses |
| [rest-api](https://github.com/ironic-org/ironic/tree/main/examples/rest-api) | Validation, versioning, serialization, compression, security, and testing |

## Running an example

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/rest-api
ironic start
```

## hello-world

The simplest possible Ironic app — a single endpoint:

```rust
#[controller("/users")]
struct UsersController;

#[routes]
impl UsersController {
    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<UserView>, HttpError> {
        // ...
    }
}
```

## rest-api

A production-style API covering:

- Request validation with `garde` and `ValidationPipe`
- API versioning (URI prefix, headers, media types)
- Response serialization with role-based field rules
- Custom exception filters
- Compression (gzip, brotli, zstd)
- CORS and security headers
- Full test suite with `TestApplication`

## What you learned

- [x] Examples demonstrate real-world patterns
- [x] `hello-world` = minimal starting point
- [x] `rest-api` = production feature showcase
