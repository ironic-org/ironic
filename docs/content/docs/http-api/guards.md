---
title: Guards
description: Authorize requests before they reach your handlers — role checks, API key validation, and custom authorization logic.
---

# Guards

## What you'll learn

- Understand where guards run in the request pipeline
- Implement custom guards with the `Guard` trait
- Use `GuardDecision::Allow` and `GuardDecision::Deny`
- Write role-based and API-key guards with real working code
- Register guards at global, controller, route, and attribute levels
- Decide when to use a guard vs middleware

## Where guards sit in the pipeline

Guards run **after middleware** but **before interceptors, extraction, pipes, and your handler**. If any guard at any level returns `Deny`, the request stops immediately — the handler is never called.

```
Request
  │
  ▼
┌──────────────┐
│  Middleware   │ ← CORS, logging, rate-limit
└──────┬───────┘
       ▼
┌──────────────┐
│   GUARDS ★    │ ← Auth checks, role checks, API keys
└──────┬───────┘   If Deny → 403 Forbidden, pipeline stops
       ▼
┌──────────────┐
│ Interceptors  │ ← Wrap handler invocation
└──────┬───────┘
       ▼
┌──────────────┐
│ Extraction +  │ ← Parse body, path, query params
│    Pipes      │
└──────┬───────┘
       ▼
┌──────────────┐
│   Handler     │ ← Your controller method
└──────────────┘
```

## The Guard trait

Every guard implements one trait with one method:

```rust
use ironic::{Guard, GuardDecision, GuardFuture, RequestContext, HttpError};
use std::{pin::Pin, future::Future};

pub trait Guard: Send + Sync + 'static {
    fn can_activate<'a>(
        &'a self,
        context: &'a mut RequestContext,
    ) -> GuardFuture<'a>;
}
```

`GuardFuture` is a type alias for `Pin<Box<dyn Future<Output = Result<GuardDecision, HttpError>> + Send + 'a>>` — all you do is return `Ok(GuardDecision::Allow)` or `Ok(GuardDecision::Deny)`. If you need to fail with an error rather than a simple deny, return `Err(...)`.

`RequestContext` gives you access to the full request (headers, method, URI, body, path params) plus a type-map extensions bag for sharing data between guards and handlers.

## GuardDecision — Allow vs Deny

```rust
pub enum GuardDecision {
    Allow,   // Continue to next guard or pipeline stage
    Deny,    // Stop immediately → 403 Forbidden with code "RF_HTTP_GUARD_DENIED"
}
```

The framework iterates through **all guards at the same level** in registration order. As soon as one returns `Deny`, the loop breaks and the framework returns a `403 Forbidden`. Every single guard must `Allow` for the request to proceed.

## Custom guard: RoleGuard

Check whether the current user has one of the required roles. This guard reads a role string that a previous guard (like an auth guard) inserted into the context's extensions:

```rust
use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};

pub struct RoleGuard {
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: &[&str]) -> Self {
        Self {
            required_roles: roles.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Guard for RoleGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let user_role = context
                .extension::<String>()
                .cloned()
                .unwrap_or_default();

            if self.required_roles.iter().any(|r| r == &user_role) {
                Ok(GuardDecision::Allow)
            } else {
                Ok(GuardDecision::Deny)
            }
        })
    }
}
```

## Custom guard: ApiKeyGuard

Validate an `x-api-key` header before allowing the request to proceed:

```rust
use ironic::{Guard, GuardDecision, GuardFuture, RequestContext, HttpError};

pub struct ApiKeyGuard {
    valid_keys: Vec<String>,
}

impl ApiKeyGuard {
    pub fn new(keys: &[&str]) -> Self {
        Self {
            valid_keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Guard for ApiKeyGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let api_key = context
                .request()
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if self.valid_keys.iter().any(|k| k == api_key) {
                Ok(GuardDecision::Allow)
            } else {
                Ok(GuardDecision::Deny)
            }
        })
    }
}
```

## Registration levels

Guards can be attached at four scopes. Global guards run first, then controller guards, then route guards, then attribute guards. All guards across all levels are flattened into a single ordered list and executed in sequence.

### Global — applied to every route

```rust
use ironic::CompiledHttpApplication;

CompiledHttpApplication::new(container, routes)
    .guard(ApiKeyGuard::new(&["sk-abc123", "sk-xyz789"]));
```

### Controller — applied to all routes in a controller

```rust
use ironic::ControllerDefinition;

ControllerDefinition::new::<MyController>("/users", provider)
    .unwrap()
    .guard(RoleGuard::new(&["admin"]));
```

### Route — applied to a single route

```rust
use ironic::{RouteDefinition, HttpMethod, handler_fn};

RouteDefinition::new(HttpMethod::GET, "/admin/dashboard", "admin_dashboard", handler_fn(admin_handler))
    .unwrap()
    .guard(RoleGuard::new(&["admin"]));
```

### Attribute macro — applied with `#[use_guard]`

On a controller struct (applies to all routes) or on individual route methods:

```rust
use ironic::prelude::*;

#[controller("/auth")]
#[derive(Injectable)]
#[use_guard(ApiKeyGuard::new(&["sk-master"]))]   // ← applies to every route
pub struct AuthController { }

#[routes]
impl AuthController {
    #[get("/me")]
    #[use_guard(AuthGuard)]                        // ← applies only to this route
    async fn me(&self) -> Result<Json<User>, HttpError> {
        // ...
    }
}
```

> `#[use_guard(...)]` accepts any expression that evaluates to a type implementing `Guard`. Multiple `#[use_guard]` attributes on the same item add guards in the order they appear.

## Guard vs Middleware — when to use which

| Concern | Use a Guard | Use Middleware |
|---------|-------------|----------------|
| Authorization check (roles, permissions) | Yes | Overkill |
| API key / token validation | Yes | Possible but guard is simpler |
| Reject with 403 | Yes (built-in) | Manual |
| Modify the response (headers, body) | No | Yes |
| Wrap the handler (timing, logging around it) | No | Yes |
| Short-circuit before extraction runs | Yes | Yes |
| Need access to extracted/parsed body | No (guards run before extraction) | Yes (middleware wraps everything) |

Rule of thumb: **if you only need to say "yes" or "no" to a request, use a guard.** If you need to observe or transform the response, use middleware.

## Guards in the request lifecycle

1. **Request arrives.** Platform adapter builds a `FrameworkRequest`.
2. **Middleware executes.** Each middleware wraps the remaining pipeline — it can short-circuit before guards run.
3. **Guards execute.** All guards (global → controller → route → attribute) run sequentially. If any returns `Deny`, the framework immediately returns `403 Forbidden` with error code `RF_HTTP_GUARD_DENIED`. No interceptor, parameter extraction, pipes, or handler code runs.
4. **Interceptors execute.** Only reached if all guards `Allow`.
5. **Handler executes.** Controller is resolved, parameters extracted and piped, handler invoked.

A guard returning `Err(_)` behaves like any pipeline error — it bypasses the standard 403 and flows into exception filters.

## Try it yourself

1. Write a `HeaderGuard` that checks for a custom `X-Tenant-Id` header
2. Insert the tenant ID into `RequestContext` extensions on `Allow`
3. Register it globally on `CompiledHttpApplication`
4. Read the tenant ID from extensions inside a handler
5. Test: request without the header → 403; with header → handler sees the tenant ID

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Guard runs but handler never sees extracted data | Guards run **before** extraction — use extensions to pass data |
| Guard registered but not running | Check registration order and that no earlier guard is denying |
| Trying to modify the response from a guard | Guards can only Allow/Deny — use middleware or interceptors instead |
| Returning `Err(...)` instead of `Deny` for auth failures | Prefer `GuardDecision::Deny` for authorization failures; `Err` is for unexpected guard errors |
| `#[use_guard]` on a struct without `Guard` trait | Make sure the type passed to `#[use_guard(...)]` implements `Guard` |

## What you learned

- [x] Guards run after middleware and before interceptors, extraction, and handlers
- [x] Implement custom guards by implementing the `Guard` trait with one async method
- [x] Return `GuardDecision::Allow` to continue or `Deny` to 403
- [x] Use `RequestContext` extensions to pass data between guards and handlers
- [x] Register guards globally, per-controller, per-route, or with `#[use_guard]`
- [x] All guards at all levels must Allow for the handler to be invoked
- [x] Choose guards for auth decisions, middleware for response transformation
