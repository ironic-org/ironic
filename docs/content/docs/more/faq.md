---
title: Troubleshooting & FAQ
description: Common errors, their causes, and how to fix them — from missing providers to scope violations and route conflicts.
---

# Troubleshooting & FAQ

## What you'll learn

- How to diagnose and fix the most common Ironic errors
- Why certain errors happen and what they mean
- Diagnostic tools built into the CLI
- Answers to frequently asked questions about Ironic

## Common Errors

### `RF_APP_MISSING_ROOT_MODULE`

**Q: The app compiles but panics at startup with "RF_APP_MISSING_ROOT_MODULE". What's wrong?**

**A:** You forgot to call `.module()` on the application builder. Ironic doesn't know which module to use as the root.

**Fix:**

```rust
// ❌ Wrong — no root module
Application::builder().run().await?;

// ✅ Correct
Application::builder()
    .module(AppModule::definition())
    .run()
    .await?;
```

---

### `RF_HTTP_ROUTE_CONFLICT`

**Q: The app panics with "RF_HTTP_ROUTE_CONFLICT". What causes this?**

**A:** Two routes in the same controller (or across controllers) registered the same method + path. Ironic detects duplicate routes at startup to prevent ambiguity.

**Fix:** Search for duplicate `#[get("/...")]`, `#[post("/...")]`, etc. with the same path. Rename one or use different HTTP methods. Run `ironic routes` to list all registered routes and spot duplicates.

---

### `RequestScopeRequired`

**Q: I'm getting "RequestScopeRequired" when calling a provider. What does it mean?**

**A:** You're trying to resolve a request-scoped provider outside of a request scope (e.g., at application startup, in a background task, or in a singleton). Request-scoped providers only exist within an HTTP request lifecycle.

**Fix:**

```rust
// ❌ Wrong — resolving request-scoped outside request context
let session = container.get::<Session>(); // panics!

// ✅ Correct — inside a controller, it's automatic
#[controller("/")]
struct MyController {
    session: Arc<Session>,  // resolved per-request
}

// ✅ Correct — manually creating a request scope
let scope = container.request_scope();
let session = scope.get::<Session>();
```

---

### `ScopeViolation`

**Q: The app panics with "ScopeViolation" at startup. Why?**

**A:** A singleton or application-scoped provider depends on a request-scoped provider. A singleton lives for the entire application lifetime, so it can't hold a reference to something that exists only per-request.

**Fix:** Either change the dependent to also be request-scoped, or pass the data differently (e.g., as a method parameter instead of a constructor dependency).

```rust
// ❌ Wrong — singleton depends on request-scoped
#[derive(Injectable)]
pub struct ReportService {
    session: Arc<Session>, // Session is request-scoped → ScopeViolation!
}

// ✅ Fix 1 — make ReportService also request-scoped
#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct ReportService {
    session: Arc<Session>,
}

// ✅ Fix 2 — pass session as a method argument instead
#[derive(Injectable)]
pub struct ReportService; // remains singleton
impl ReportService {
    pub fn generate(&self, session: &Session) -> Report { /* ... */ }
}
```

---

### "Service not found / not registered"

**Q: My service compiles but at runtime I get "service not found" or it's not injected. What's missing?**

**A:** The provider wasn't registered in any module's `providers` array. Ironic uses explicit registration — it doesn't auto-discover providers.

**Fix:** Add the provider to your module:

```rust
#[module(
    controllers = [MyController],
    providers = [MyService]   // ← add this
)]
pub struct MyModule;
```

Also check that the module importing `MyModule` actually imports it via `modules = [...]`.

---

### "Circular dependency detected: A → B → A"

**Q: I get a circular dependency error. How do I fix it?**

**A:** Two or more providers depend on each other, forming a cycle the DI container can't resolve. Common patterns that cause this: forward references, coupled services.

**Fix:** Refactor by extracting shared logic into a third service:

```rust
// ❌ Circular: A → B → A
#[derive(Injectable)]
pub struct ServiceA { service_b: Arc<ServiceB> }
#[derive(Injectable)]
pub struct ServiceB { service_a: Arc<ServiceA> }

// ✅ Break the cycle with a shared dependency
#[derive(Injectable)]
pub struct ServiceA { shared: Arc<SharedLogic> }
#[derive(Injectable)]
pub struct ServiceB { shared: Arc<SharedLogic> }
```

---

### `RF_APP_INVALID_ADDRESS`

**Q: The app panics with "RF_APP_INVALID_ADDRESS". What format does the server address expect?**

**A:** The address string couldn't be parsed as a valid `SocketAddr`. The expected format is `IP:PORT`.

**Fix:**

```rust
// ❌ Wrong
configure!(SERVER_ADDRESS = "localhost:3000");

// ✅ Correct
configure!(SERVER_ADDRESS = "127.0.0.1:3000");

// ✅ Also valid — bind to all interfaces
configure!(SERVER_ADDRESS = "0.0.0.0:8080");
```

---

### "Forgetting `Arc<T>` for injected dependencies"

**Q: My struct compiles but DI fails silently. What did I miss?**

**A:** Injectable struct fields holding dependencies must be wrapped in `Arc<T>`. Ironic shares providers across threads via `Arc`, and the DI container won't connect them without it.

**Fix:**

```rust
// ❌ Wrong — missing Arc
#[derive(Injectable)]
pub struct OrderService {
    payment: PaymentService,  // won't be injected!
}

// ✅ Correct
#[derive(Injectable)]
pub struct OrderService {
    payment: Arc<PaymentService>,
}
```

---

### App unreachable in Docker

**Q: My Ironic app works locally but is unreachable inside a Docker container. Why?**

**A:** `SERVER_HOST` defaults to `127.0.0.1` (localhost), which only accepts connections from the same container. In Docker, you need to bind to all interfaces.

**Fix:** Set the host to `0.0.0.0`:

```bash
# Dockerfile
ENV SERVER_HOST=0.0.0.0

# Or via docker run
docker run -e SERVER_HOST=0.0.0.0 my-app
```

---

## Diagnostic Tips

| Tool | What it does |
|------|-------------|
| `RUST_LOG=debug` | Enables verbose framework logging (module resolution, route registration, provider creation) |
| `ironic doctor` | Checks your environment — Rust version, toolchain, common misconfigurations |
| `ironic routes` | Lists all registered routes with their HTTP method, path, and handler |
| `ironic graph` | Prints the full dependency graph of your application's modules and providers |

Run diagnostics before debugging runtime errors:

```bash
RUST_LOG=debug ironic start
ironic routes
ironic graph
```

---

## FAQ

### Can I use Ironic without Axum?

Currently, Axum is the only platform adapter. Support for other HTTP frameworks may be added in the future.

### How do I add a database connection?

Enable the database feature and create an eager singleton wrapping a connection pool:

```rust
#[derive(Injectable)]
#[scope(singleton, eager)]
pub struct Database {
    pool: sqlx::PgPool,
}
```

See [Database Integrations](../data-auth/database-integrations) for full examples with PostgreSQL, MySQL, MongoDB, and Redis.

### How do I handle file uploads?

Use the ready-made module scaffolded by the CLI:

```bash
ironic generate rr file-upload
```

This creates a controller with multipart parsing, size limits, and storage handling.

### How do I add authentication?

Use `ironic generate rr auth` for a pre-built authentication module, or enable the auth feature in `Cargo.toml`:

```toml
[dependencies]
ironic = { version = "0", features = ["authentication"] }
```

See [Authentication](../data-auth/authentication) for JWT, OAuth, and session-based setups.

---

## What you learned

- [x] 8 common errors with causes and code fixes
- [x] How to diagnose problems with `RUST_LOG=debug`, `ironic doctor`, `ironic routes`, and `ironic graph`
- [x] Why `Arc<T>` is required for injected dependencies
- [x] How to configure Docker containers to accept connections
- [x] How to add databases, file uploads, and authentication
