---
title: Request Lifecycle
description: How an HTTP request flows through the Ironic pipeline — from socket to response.
---

# Request Lifecycle

Every HTTP request passes through a well-defined pipeline. Understanding this flow helps you choose where to add your logic.

## The full pipeline

```
┌──────────┐
│  Client  │
└────┬─────┘
     │ HTTP Request
     ▼
┌──────────────────┐
│  Platform Server │  ← Axum receives the TCP connection
│  (Axum/Hyper)    │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Tower Layers    │  ← Compression, CORS, logging, rate limiting
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Router Match    │  ← Matches method + path to a controller
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Guards          │  ← Authorization checks
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Interceptors    │  ← Pre-handler: validation, logging
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Pipes           │  ← Parameter transformation
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Handler         │  ← Your controller method runs
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Result          │  ← Result<T, E> is converted to response
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Exception Filter│  ← Catches errors (if Result is Err)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Interceptors    │  ← Post-handler: response transformation
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Response        │  ← JSON / HTML / Redirect / error body
└──────┬───────────┘
       │
       ▼
┌──────────┐
│  Client  │
└──────────┘
```

## Step by step

### 1. Platform server

Axum receives the raw HTTP request from the network.

### 2. Tower middleware

Request passes through tower layers configured on the router:

```rust
AxumAdapter::new()
    .configure_router(|router| {
        router
            .layer(CompressionLayer::new())
            .layer(CorsLayer::new(config))
            .layer(MetricsLayer::new(MetricsConfig::default()))
    })
```

### 3. Router match

The router matches the HTTP method and path to a registered controller route. If no match is found, a 404 is returned immediately.

### 4. Guards

Guards check authorization before the handler runs:

```rust
#[guard(AuthGuard)]
#[get("/profile")]
async fn get_profile(&self) -> Json<User> {
    // Only runs if AuthGuard returns Allow
}
```

If a guard returns `Deny`, the request is rejected with 401/403 and the handler never runs.

### 5. Interceptors (pre-handler)

Interceptors run before the handler. They can:
- Validate request headers
- Attach metadata to the request context
- Transform the request
- Short-circuit with an error

### 6. Pipes

Pipes transform and validate parameters:

```rust
async fn get(&self, id: PathParameter<ParseIntPipe>) -> Json<User> {
    // id is validated as an integer before the handler runs
}
```

### 7. Handler

Your controller method executes:

```rust
async fn create(&self, body: JsonBody<CreateUser>) -> Result<Json<User>, HttpError> {
    let user = self.service.create(body.0).await?;
    Ok(Json(user))
}
```

### 8. Response conversion

The handler's return value is converted to an HTTP response via `IntoResponse`:

- `Json<T>` → JSON response with `Content-Type: application/json`
- `Html<T>` → HTML response
- `Redirect` → 301/302 redirect
- `HttpError` → Structured error response
- `Result<T, E>` → Success or error response

### 9. Exception filters (on error)

If the handler returns an `Err`, exception filters process the error:

```rust
struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(&self, error: &HttpError, ctx: &FilterContext) -> Option<HttpResponse> {
        if error.status == StatusCode::NOT_FOUND {
            Some(HttpResponse::new(404).body("Custom 404 page"))
        } else {
            None
        }
    }
}
```

### 10. Interceptors (post-handler)

Interceptors run after the handler to transform the response (e.g., adding headers, wrapping in an envelope).

## Key ordering

```
Middleware → Router → Guards → Interceptors → Pipes → Handler → ExceptionFilter → Interceptors → Response
```

Each stage can short-circuit the pipeline by returning early:
- **Middleware**: Returns 503 for rate limiting
- **Guards**: Returns 401/403 for unauthorized access
- **Interceptors**: Returns validation error
- **Pipes**: Returns parse error
- **Handler**: Returns business logic error
- **ExceptionFilter**: Returns custom error response
