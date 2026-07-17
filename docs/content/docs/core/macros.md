---
title: Macros & Attributes
description: Complete reference for every Ironic macro and attribute — Injectable, Module, controller, route methods, parameter extractors, guards, and interceptors.
---

# Macros & Attributes

## What you'll learn

- Every macro and attribute Ironic provides — and what each one does
- How `#[derive(Module)]` and `#[derive(Injectable)]` wire up the DI container
- How to declare routes with `#[get]`, `#[post]`, and the full HTTP verb set
- How to extract path params, query strings, JSON bodies, and headers
- How to attach guards and interceptors with `#[guard]` / `#[interceptor]`
- How `#[ironic::main]` and `#[web_socket_gateway]` work

---

## Module macros

### `#[derive(Module)]` + `#[module(...)]`

Generates a static module definition consumed by the application builder.

```rust
#[derive(Module)]
#[module(
    imports     = [HealthModule],
    providers   = [UserService],
    controllers = [UserController],
    exports     = [UserService],
)]
pub struct UserModule;
```

| Field | Purpose |
|-------|---------|
| `imports` | Other modules whose exports this module needs |
| `providers` | Injectable services registered inside this module |
| `controllers` | Controllers whose routes belong to this module |
| `exports` | Providers exposed to modules that import this one |

> Every module consumed by your app **must** appear in `imports` of the root module. Missing imports are caught at compile time.

**Common mistakes:**
- Forgetting to add a new module to `imports` of `AppModule` — routes silently won't register.
- Exporting a service but not listing it in `providers` first (exports references providers).

---

## Service macros

### `#[derive(Injectable)]` + `#[injectable(...)]`

Generates a `provider_definition()` method so the DI container can construct and resolve the service.

```rust
#[derive(Injectable)]
#[injectable(scope = "transient", eager)]
pub struct IdGenerator;
```

| Option | Values | Default | Effect |
|--------|--------|---------|--------|
| `scope` | `"singleton"`, `"transient"`, `"request"` | `"singleton"` | Controls how many instances exist |
| `eager` | (flag) | off | Construct at bootstrap instead of on first use |
| `optional` | (flag) | off | Dependencies resolve to `None` when not registered |

**Common mistakes:**
- Omitting `#[derive(Injectable)]` on a struct listed in `providers` — the compiler will complain about a missing `provider_definition()`.
- Using `scope = "request"` on a dependency injected into a singleton — causes a runtime `ScopeViolation`.

---

## Controller macros

### `#[controller("/prefix")]` + `#[routes]`

`#[controller("/prefix")]` generates a `controller_definition()` with the path prefix. `#[routes]` collects HTTP method attributes from the `impl` block and registers them as route definitions.

```rust
#[controller("/products")]
#[derive(Injectable)]
pub struct ProductsController {
    products: Arc<ProductService>,
}

#[routes]
impl ProductsController {
    #[get("/")]
    async fn list(&self) -> Result<Json<Vec<Product>>, HttpError> { ... }

    #[get("/:id")]
    async fn show(&self, #[param] id: u64) -> Result<Json<Product>, HttpError> { ... }
}
```

`#[routes]` must appear on the `impl` block — not on individual methods. Every `#[get]`, `#[post]`, etc. inside the block is collected.

**Common mistakes:**
- Placing `#[routes]` on the struct instead of the `impl` block.
- Forgetting `#[routes]` entirely — route methods compile but never register.
- Using `#[controller]` without `#[derive(Injectable)]` — the controller must be injectable to receive dependencies.

---

## Route method macros

Every standard HTTP verb is available as an attribute macro. Each accepts a path string argument:

```rust
#[get("/:id")]           // GET /prefix/42
#[post("/")]             // POST /prefix
#[put("/:id")]           // PUT /prefix/42
#[delete("/:id")]        // DELETE /prefix/42
#[patch("/:id")]         // PATCH /prefix/42
#[head("/")]             // HEAD /prefix
#[options("/")]          // OPTIONS /prefix
```

All verbs support path parameters with the `:` syntax (e.g. `"/users/:user_id/posts/:post_id"`). The path is appended to the controller's prefix from `#[controller("/prefix")]`.

> Route handlers must be **async** and return `Result<T, HttpError>` where `T` implements `IntoResponse`.

**Common mistakes:**
- Returning a bare type instead of `Result<T, HttpError>` — the framework won't accept it.
- Forgetting the leading `/` in the path — `#[get("items")]` instead of `#[get("/items")]`.

---

## Parameter extractors

Extract data from incoming requests with parameter attributes on handler arguments:

```rust
#[post("/users")]
async fn create(
    &self,
    #[body]   payload: CreateUserDto,   // JSON body → deserialized
    #[query]  tenant: String,           // ?tenant=acme
    #[header("x-api-key")] key: String, // Request header
) -> Result<Json<User>, HttpError> { ... }

#[get("/users/:id")]
async fn show(
    &self,
    #[param] id: u64,                   // /users/42 → id = 42
) -> Result<Json<User>, HttpError> { ... }
```

| Attribute | Source | Example |
|-----------|--------|---------|
| `#[param]` | Path segment (`/:name`) | `/users/42` → `42` |
| `#[body]` | Request body (JSON) | `POST` with `{"name":"Alice"}` |
| `#[query]` | Query string | `?filter=active` |
| `#[header("name")]` | Request header | `Authorization: Bearer ...` |

**Common mistakes:**
- Using `#[param]` on a route that doesn't declare the corresponding `:param` in its path — runtime panic.
- Forgetting `#[body]` when you need the JSON payload — the argument gets the wrong extractor.

---

## Pipeline attributes

### `#[guard]`, `#[interceptor]`, `#[middleware]`, and `#[exception]`

Attach guards, interceptors, middleware, and exception filters at the controller level:

```rust
#[controller("/admin")]
#[guard(AuthGuard)]
#[interceptor(LoggingInterceptor)]
#[middleware(RateLimitMiddleware::new(100))]
#[exception(NotFoundFilter)]
#[derive(Injectable)]
pub struct AdminController { ... }
```

`#[guard]`, `#[interceptor]`, and `#[middleware]` also work on individual route methods. `#[exception]` is controller-level only — use `.exception_filter(...)` on route definitions for per-route filtering.

Guards run **before** the handler and can short-circuit with an error. Interceptors **wrap** handler execution (before + after). Middleware wraps the full request lifecycle. Exception filters catch errors after the handler fails.

**Common mistakes:**
- Forgetting to register the guard/interceptor as a provider — they must be in `providers` to be resolved.
- Attaching `#[guard]` to a method expecting a different guard signature — type mismatch at compile time.

---

## Entry point

### `#[ironic::main]`

Wraps an `async fn main()` with Ironic's Tokio runtime and bootstrap machinery:

```rust
#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build().await.unwrap()
        .listen("127.0.0.1:3000").await.unwrap();
}
```

The macro sets up the multi-threaded Tokio runtime automatically. It accepts **no arguments** and requires the function to be `async`.

**Common mistakes:**
- Passing arguments to `#[ironic::main(...)]` — not supported, the macro rejects them.
- Using `#[ironic::main]` on a non-async function — compile error.

---

## WebSocket

### `#[web_socket_gateway("/path")]`

Declares a WebSocket endpoint:

```rust
#[web_socket_gateway("/chat")]
pub struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {}", payload))
    }
}
```

The macro registers the WebSocket upgrade handler at the given path. Message handlers inside `#[routes]` use `#[subscribe_message("event")]` to listen for specific message types.

**Common mistakes:**
- Using `#[controller]` instead of `#[web_socket_gateway]` for WebSocket endpoints — they are different protocols.
- Forgetting `#[subscribe_message]` on handler methods — they won't receive incoming messages.

---

## Try it yourself

1. Create a `#[controller("/api")]` with a `#[get("/health")]` route
2. Add `#[guard]` that checks for a custom header
3. Extract a `#[query]` parameter and echo it back in the response
4. Register the controller, guard, and module in `AppModule`

## What you learned

- [x] `#[derive(Module)]` groups providers, controllers, imports, and exports
- [x] `#[derive(Injectable)]` with `#[injectable(...)]` controls scope and eager init
- [x] `#[controller("/prefix")]` + `#[routes]` declare HTTP endpoints
- [x] `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`, `#[head]`, `#[options]` map HTTP verbs
- [x] `#[param]`, `#[body]`, `#[query]`, `#[header]` extract request data
- [x] `#[guard]` and `#[interceptor]` plug into the request pipeline
- [x] `#[ironic::main]` sets up the async runtime
- [x] `#[web_socket_gateway]` + `#[subscribe_message]` handle WebSocket connections

## Next steps

Understand how services are created and shared:

→ [Service Lifetimes](./lifetimes)
