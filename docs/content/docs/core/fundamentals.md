---
title: Fundamentals
description: Learn the 4 building blocks of Ironic — Modules, Controllers, Services, and Dependency Injection.
---

# Fundamentals

## What you'll learn

- What Modules, Controllers, and Services are — and how they fit together
- How Dependency Injection (DI) works (and why you want it)
- How to write routes that handle real requests
- How to compose modules into a full application

## The big picture

An Ironic app is like a **Russian doll** — each layer wraps the one inside it:

```
┌─────────────────────────────────────────┐
│              AppModule                  │  ← Top-level: imports everything
│  ┌───────────────────────────────────┐  │
│  │         ProductsModule            │  │  ← Feature: groups related code
│  │  ┌──────────┐  ┌───────────────┐  │  │
│  │  │Controller│  │   Service     │  │  │
│  │  │ (routes) │◄─│(business logic)│  │  │  ← Inside: the actual code
│  │  └──────────┘  └───────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### The 4 building blocks

| Building Block | What it does | Real-world analogy |
|---------------|-------------|-------------------|
| **Module** | Groups related code together | A department in a company |
| **Controller** | Handles HTTP requests (GET, POST, etc.) | The reception desk |
| **Service** | Contains business logic | The workers in the back office |
| **DI (Dependency Injection)** | Automatically connects Services to Controllers | The company org chart |

## Building Block 1: Services

A **Service** is where your business logic lives. It's a plain Rust struct marked with `#[derive(Injectable)]`:

```rust
use ironic::prelude::*;

#[derive(Injectable)]          // ← Makes it injectable via DI
pub struct CalculatorService;  // ← A unit struct (no fields needed)

impl CalculatorService {
    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn greet(&self, name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

> **Key idea:** Services are "dumb" — they don't know about HTTP, routes, or JSON. They just do work and return results. This makes them easy to test.

### Services with dependencies

A Service can depend on other Services. Just add them as fields:

```rust
#[derive(Injectable)]
pub struct OrderService {
    payment: std::sync::Arc<PaymentService>,  // ← Dependencies go here
    inventory: std::sync::Arc<InventoryService>,
}
// Ironic automatically provides PaymentService and InventoryService!
```

> **`Arc` is required** — all injected dependencies must be wrapped in `Arc<T>`. This is how Ironic shares services safely across threads. Think of it as a "shared reference."

## Building Block 2: Controllers

A **Controller** handles HTTP requests. It receives the request, calls a Service, and returns a response:

```rust
#[controller("/math")]        // ← All routes in this controller start with /math
#[derive(Injectable)]
pub struct MathController {
    calc: std::sync::Arc<CalculatorService>,  // ← Injected automatically!
}

#[routes]                     // ← Start defining routes
impl MathController {
    // GET /math/add?x=5&y=3 → 8
    #[get("/add")]
    async fn add(&self, #[query] x: i32, #[query] y: i32) -> Result<String, HttpError> {
        Ok(self.calc.add(x, y).to_string())
    }

    // GET /math/greet/Alice → "Hello, Alice!"
    #[get("/greet/:name")]
    async fn greet(&self, #[param] name: String) -> Result<String, HttpError> {
        Ok(self.calc.greet(&name))
    }
}
```

### Route parts explained

```
#[get("/greet/:name")]
        │         │
        │         └── Path parameter (dynamic part of URL)
        └── HTTP method + path pattern

GET /math/greet/Alice
    │    │      │
    │    │      └── :name = "Alice"
    │    └── Controller prefix (@controller("/math"))
    └── HTTP verb from #[get]
```

| Annotation | What it does | Example URL |
|-----------|-------------|-------------|
| `#[get("/path")]` | Handle GET requests | `GET /math/path` |
| `#[post("/path")]` | Handle POST requests | `POST /math/path` |
| `#[put("/path")]` | Handle PUT requests | `PUT /math/path` |
| `#[delete("/path")]` | Handle DELETE requests | `DELETE /math/path` |

### Extracting data from requests

| Parameter | What it extracts | Example |
|----------|-----------------|---------|
| `#[param] id: u64` | Path parameter (`/:id`) | `/items/42` → `id = 42` |
| `#[query] filter: String` | Query string (`?filter=...`) | `/items?filter=active` |
| `#[body] data: T` | JSON request body | `POST /items` with JSON body |
| `#[header("x-key")] val: String` | HTTP header | Reads `x-key` header |

## Building Block 3: Modules

A **Module** groups related Controllers and Services together:

```rust
#[derive(Module)]
#[module(
    providers = [CalculatorService],      // ← Services go here
    controllers = [MathController],       // ← Controllers go here
)]
pub struct MathModule;
```

> Think of a Module as a **shipping container** — it packages everything needed for one feature.

### Composing modules

The root module (`AppModule`) imports all other modules:

```rust
#[derive(Module)]
#[module(imports = [MathModule, ProductsModule, HealthModule])]
pub struct AppModule;
```

Every module your app uses must be listed in `imports`. The framework validates this at compile time — if you forget one, the compiler tells you!

### Module visibility

| Declaration | What it means |
|------------|---------------|
| `providers = [Service]` | This Service is available inside this module |
| `controllers = [Controller]` | This Controller handles routes in this module |
| `imports = [OtherModule]` | Use everything exported by OtherModule |
| `exports = [Service]` | Make this Service available to modules that import us |

## Building Block 4: Dependency Injection

**Dependency Injection (DI)** means you don't create dependencies yourself — the framework creates them and hands them to you.

### Without DI (manual):

```rust
// You have to create everything yourself
let payment = PaymentService::new();
let inventory = InventoryService::new();
let orders = OrderService::new(payment, inventory);
// What if PaymentService needs a DatabaseService?
// Now you need to create that too... and the chain keeps growing
```

### With DI (automatic):

```rust
#[derive(Injectable)]
pub struct OrderService {
    payment: Arc<PaymentService>,      // ← Ironic creates this for you
    inventory: Arc<InventoryService>,  // ← And this too
}
// That's it! Ironic figures out the entire dependency graph.
```

> **Why this matters:** When your app grows to 50 services with complex dependencies, DI keeps everything organized. Without it, you'd be manually wiring hundreds of dependencies in `main.rs`.

### Service lifetimes (scopes)

| Scope | How long it lives | When to use |
|-------|-------------------|-------------|
| **Singleton** (default) | Created once, shared everywhere | Database connections, config, caches |
| **Transient** | New copy every time it's requested | Lightweight stateless helpers |
| **Request** | Created per HTTP request, destroyed after | User session data, request tracking |

```rust
#[derive(Injectable)]
#[injectable(scope = "transient")]  // ← Each injection gets a fresh copy
pub struct IdGenerator;
```

## The complete picture

Here's how a request flows through all 4 building blocks:

```
HTTP Request: GET /products/42
         │
         ▼
    ┌─────────┐
    │ AppModule│  ← Top-level: routes request to ProductsModule
    └────┬────┘
         ▼
    ┌──────────────┐
    │ProductsModule │  ← Feature module: owns the controller
    └────┬─────────┘
         ▼
    ┌──────────────────┐
    │ ProductsController│  ← Handles GET /:id, extracts id=42
    │  #[get("/:id")]  │
    └────┬─────────────┘
         │ calls ↓
    ┌────────────────┐
    │ ProductsService │  ← Business logic: finds product #42
    │  find(42)       │
    └────┬───────────┘
         ▼
    Ok(Json(product))  ← Response: JSON back to the client
```

## Try it yourself

1. Create a `CalculatorService` with `add` and `multiply` methods
2. Create a `MathController` at `/math` with endpoints for both
3. Register them in a `MathModule`
4. Import `MathModule` into `AppModule`
5. Test with curl: `curl http://localhost:3000/math/add?x=10\&y=5`

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting `Arc<T>` on dependencies | All injected fields must be `Arc<Service>` |
| Service not in `providers` | Add every Service to `providers = [...]` |
| Controller not in `controllers` | Add every Controller to `controllers = [...]` |
| Module not in `imports` | Add the module to `AppModule`'s `imports` |
| `#[derive(Injectable)]` missing on Service | Every injectable struct needs `#[derive(Injectable)]` |

## What you learned

- [x] Services contain business logic and are marked with `#[derive(Injectable)]`
- [x] Controllers handle HTTP requests and define `#[routes]`
- [x] Modules group related code with `providers`, `controllers`, and `imports`
- [x] Dependency Injection automatically connects Services to Controllers
- [x] The request flows: Module → Controller → Service → Response
- [x] All injected dependencies need `Arc<T>`

## Next steps

Now that you understand the building blocks, learn how the CLI helps you scaffold code faster:

→ [CLI Reference](./cli)
