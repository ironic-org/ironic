---
title: Getting Started
description: Install Ironic, create your first project, and have a running API in under 60 seconds.
---

# Getting Started

## What you'll learn

- Install the Ironic CLI
- Create a new project
- Run it locally
- Generate your first API endpoint
- Understand the project structure

## The big picture

Building an API with Ironic is like assembling furniture with instructions — the CLI does the heavy lifting, and you focus on what makes your app unique.

```
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│  ironic new │ ──► │ ironic start │ ──► │ ironic gen    │ ──► Your API!
└─────────────┘     └──────────────┘     └───────────────┘
   Creates project    Runs the server     Adds endpoints
```

## Step 1: Install the CLI

Open your terminal and run:

```bash
cargo install ironic
```

This installs the `ironic` command globally. Verify it worked:

```bash
ironic --version
# → ironic 0.1.7
```

> **Troubleshooting:** If you get "command not found", make sure `~/.cargo/bin` is in your `PATH`. Add `export PATH="$HOME/.cargo/bin:$PATH"` to your `~/.zshrc` or `~/.bashrc`.

## Step 2: Create a project

```bash
ironic new my-first-api
cd my-first-api
```

This creates a folder called `my-first-api` with everything you need. Let's look at what's inside:

```
my-first-api/
├── Cargo.toml          ← Dependencies (ironic + serde)
├── ironic.toml         ← Project configuration
└── src/
    ├── main.rs         ← Entry point — starts the server
    ├── app.rs          ← Root module — imports everything
    └── modules/
        └── mod.rs      ← Module registry — empty for now
```

## Step 3: Run it

```bash
ironic start
```

You'll see:

```
🚀 Server listening on http://127.0.0.1:3000
```

Open `http://localhost:3000/health` in your browser. You should see:

```json
{"status": "ok"}
```

**Congratulations!** Your first Ironic API is running. Press `Ctrl+C` to stop it.

## Step 4: Add your first endpoint

Let's create a "products" API with the resource generator:

```bash
ironic generate resource products
```

This creates 11 files! But don't worry — here's what matters:

```
src/modules/products/
├── mod.rs                  ← Module wiring
├── tests/                  ← Auto-generated tests!
│   ├── unit.rs             ← Fast tests (no HTTP)
│   └── integration.rs      ← Full HTTP tests
├── controller/
│   └── products_controller.rs  ← Routes (GET, POST, PUT, DELETE)
├── services/
│   └── products_service.rs     ← Business logic
├── dto/
│   ├── create_products_dto.rs  ← Input validation
│   └── update_products_dto.rs
└── entities/
    └── products.rs             ← Data model
```

> **Think of it like a restaurant:**
> - **Controller** = the waiter (takes orders, brings food)
> - **Service** = the chef (does the actual cooking)
> - **DTO** = the menu (what customers can order)
> - **Entity** = the recipe (what the food actually is)

Run the server again:

```bash
ironic start
```

Now visit `http://localhost:3000/products` — you'll get a response from your generated controller!

## Step 5: Run the tests

Your generated code comes with tests:

```bash
ironic test
```

You'll see output like:

```
test products::tests::unit::service_has_the_correct_name ... ok
test products::tests::integration::list_endpoint_returns_empty ... ok
```

All green! Your API is tested and working.

## Step 6: Write real logic

Open `src/modules/products/controller/products_controller.rs`. You'll see a `list` method that returns the service name. Let's make it return actual data.

First, add some data to the service (`services/products_service.rs`):

```rust
use std::sync::Mutex;

// Add a simple in‑memory store
static ITEMS: Mutex<Vec<String>> = Mutex::new(Vec::new());

impl ProductsService {
    pub fn list(&self) -> Vec<String> {
        ITEMS.lock().unwrap().clone()
    }

    pub fn add(&self, name: String) {
        ITEMS.lock().unwrap().push(name);
    }
}
```

Then update the controller:

```rust
#[post]
async fn create(&self, #[body] dto: CreateProductsDto) -> Result<(HttpStatus, Json<String>), HttpError> {
    self.service.add(dto.name);
    Ok((HttpStatus::CREATED, Json("Created!".into())))
}

#[get]
async fn list(&self) -> Result<Json<Vec<String>>, HttpError> {
    Ok(Json(self.service.list()))
}
```

Now `POST /products` adds items and `GET /products` lists them. Restart the server and test it:

```bash
curl -X POST http://localhost:3000/products -H "Content-Type: application/json" -d '{"name":"Laptop"}'
curl http://localhost:3000/products
# → ["Laptop"]
```

## Try it yourself

1. Create a new project called `bookstore-api`
2. Generate a resource called `books`
3. Add a `GET /books/:id` route that returns a specific book
4. Run `ironic test` and make sure the auto-generated tests pass

## Common mistakes

| Mistake | Fix |
|---------|-----|
| `ironic` command not found | Add `~/.cargo/bin` to your `PATH` |
| Port already in use | Change the port in `src/main.rs` to `3001` |
| "module not found" error | Run `ironic generate resource` again — the CLI auto-registers modules |
| Tests fail with "unresolved import" | Make sure you ran `ironic generate resource` first |

## What you learned

- [x] Installed `ironic` CLI globally
- [x] Created a new project with `ironic new`
- [x] Ran the development server with `ironic start`
- [x] Generated a full resource with controller, service, DTOs, entity, and tests
- [x] Wrote real business logic and tested it with curl
- [x] Ran auto-generated tests with `ironic test`

## Next steps

Now that you have a working API, learn the **4 building blocks** of Ironic:

→ [Fundamentals: Modules, Controllers, Services & DI](./fundamentals)
