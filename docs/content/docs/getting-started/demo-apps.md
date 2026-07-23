---
title: Demo Apps
description: Example applications built with Ironic — from simple CRUD to full-stack production demos.
---

# Demo Apps

The Ironic project maintains several demo applications that showcase different aspects of the framework. These are published on GitHub and can be used as reference implementations or starting points.

## Quick start

```bash
# Clone the examples repo
git clone https://github.com/ironic-org/ironic
cd ironic/examples

# Run the blog API demo
cd blog
cargo run
```

## Available demos

### Blog API

A complete CRUD API for a blog with users, posts, and comments.

**Features demonstrated:**
- Controllers and routes
- Dependency injection with services and repositories
- SQLx database integration (PostgreSQL)
- JWT authentication
- Input validation with pipes
- Error handling with exception filters
- OpenAPI documentation

```bash
cd examples/blog
# Set up database
cp .env.example .env
# Run migrations
cargo run -- migrate
# Start server
cargo run
```

### Chat application

A real-time chat application using WebSocket gateways.

**Features demonstrated:**
- WebSocket gateways with rooms
- Message broadcasting
- Authentication via JWT
- Typed message handling
- Connection lifecycle management

### E-commerce API

A more complex example with orders, products, inventory, and payments.

**Features demonstrated:**
- Modular architecture (6 modules)
- Redis caching
- Background job processing
- Event-driven architecture with event bus
- Distributed sagas for order processing
- Rate limiting and security middleware

## Creating a demo from scratch

You can also use the CLI to create a new project and add features step by step:

```bash
# Create project
ironic new my-demo
cd my-demo

# Add resources
ironic gen resource user
ironic gen resource post
ironic gen resource comment

# Add auth
ironic gen auth jwt

# Run it
ironic start
```

## Publishing your own app

When you're ready to publish your Ironic application:

1. **Build for release**: `cargo build --release`
2. **Optimize**: Enable LTO and codegen-units in `Cargo.toml`
3. **CI/CD**: Use the included GitHub Actions workflows
4. **Deployment**: Build a Docker image:

```dockerfile
FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-app /app/my-app
EXPOSE 3000
CMD ["/app/my-app"]
```

5. **Monitor**: The metrics endpoint at `GET /metrics` integrates with Prometheus
