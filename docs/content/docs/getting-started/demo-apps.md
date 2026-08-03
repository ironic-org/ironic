---
title: Demo Apps
description: Example applications built with Ironic — from simple CRUD to full-stack production demos.
---

# Demo Apps

The Ironic project maintains a reference application — the blog API in `examples/blog` — that showcases the framework's production features. It can be used as a reference implementation or a starting point.

## Quick start

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/blog
cargo run
```

See [The Blog API demo](#the-blog-api-demo) below for database setup and migration steps.

## The Blog API demo

The `examples/blog` application is a complete CRUD API for a blog with users, posts, and comments. It is the reference implementation for the framework's production features.

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
