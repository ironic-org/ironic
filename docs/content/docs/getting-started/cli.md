---
title: CLI Reference
description: Master the Ironic command-line tools — create, generate, run, test, and inspect your application.
---

# CLI Reference

## What you'll learn

- Every CLI command and what it does
- Generator commands for scaffolding code
- Migration commands for database schema management
- Project inspection tools
- Doctor command for debugging

---

## Project commands

| Command | What it does |
|---------|-------------|
| `ironic new <name>` | Create a new HTTP project |
| `ironic new <name> --graphql` | Create a new GraphQL project |
| `ironic new .` | Create project in the current directory |
| `ironic start` | Run the server (`cargo run`) |
| `ironic start -p <name>` | Run a specific app in a monorepo (`cargo run -p <name>`) |
| `ironic dev` | Run with hot reload — auto-restarts on file changes |
| `ironic dev -p <name>` | Dev mode for a monorepo app — watches `apps/<name>/src/` |
| `ironic build` | Build the project (`cargo build`) |
| `ironic test` | Run tests (`cargo test`) |

## Generator commands

| Command | Alias | Creates |
|---------|-------|---------|
| `ironic generate resource <name>` | `g res` | Full module with controller, service, DTOs, entity, and tests |
| `ironic generate ready-resource auth` | `g rr auth` | Production-ready auth module (JWT, OAuth, RBAC) |
| `ironic generate ready-resource file-upload` | `g rr file-upload` | File upload module (local, S3, R2 backends) |
| `ironic generate ready-resource email` | `g rr email` | Email module (SMTP, SES, SendGrid, Mailgun) |
| `ironic generate module <name>` | `g mo` | Module shell only |
| `ironic generate controller <name>` | `g co` | Controller inside a module |
| `ironic generate service <name>` | `g s` | Service inside a module |
| `ironic generate decorator <name>` | `g d` | Custom parameter decorator |
| `ironic generate filter <name>` | `g f` | Exception filter |
| `ironic generate guard <name>` | `g gu` | Auth guard |
| `ironic generate middleware <name>` | `g mi` | Middleware |
| `ironic generate pipe <name>` | `g pi` | Parameter pipe |
| `ironic generate provider <name>` | `g pr` | Injectable provider |
| `ironic generate app <name>` | `g a` | New HTTP microservice in the monorepo |
| `ironic generate app <name> --grpc` | `g a --grpc` | New gRPC microservice with tonic + DI |
| `ironic generate app <name> --graphql` | `g a --graphql` | New GraphQL microservice with async-graphql |

## Ready Resource Generators

These generate production-ready modules with minimal setup. Run them from your project root.

### Auth Module

```bash
# Full auth (JWT + OAuth + passwords + sessions + RBAC)
ironic generate ready-resource auth

# Basic auth (passwords + sessions only)
ironic generate ready-resource auth-basic

# JWT only
ironic generate ready-resource auth-jwt

# OAuth only (Google, GitHub)
ironic generate ready-resource auth-oauth
```

**Output** — creates `src/modules/auth/` with:
```
auth/
├── mod.rs           # AuthModule (auto-registered)
├── auth.controller.rs  # Login, register, refresh endpoints
├── auth.service.rs  # Business logic
├── password.service.rs # Password hashing (argon2)
├── jwt.service.rs   # JWT token management (if applicable)
├── guards/
│   ├── mod.rs
│   ├── auth.guard.rs    # JWT/session validation
│   └── role.guard.rs    # RBAC authorization
├── decorators/
│   ├── mod.rs
│   ├── current_user.rs  # Extract user from request
│   └── roles.rs         # Role-based access
├── dto/
│   ├── mod.rs
│   ├── register.dto.rs
│   ├── login.dto.rs
│   └── token.response.rs
├── entities/
│   ├── mod.rs
│   └── user.rs
├── services/
│   └── mod.rs
└── tests/
    ├── mod.rs
    ├── unit.rs
    └── integration.rs
```

**Dependencies auto-added**: `argon2`, `jsonwebtoken`, `oauth2`, `getrandom`

### File Upload Module

```bash
ironic generate ready-resource file-upload
```

**Output** — creates `src/modules/file-upload/` with:
```
file-upload/
├── mod.rs
├── file-upload.controller.rs  # Upload, list, delete endpoints
├── file-upload.service.rs     # Storage logic
├── adapters/
│   ├── mod.rs
│   ├── local.rs          # Local filesystem
│   ├── s3.rs             # AWS S3
│   ├── r2.rs             # Cloudflare R2
│   ├── azure.rs          # Azure Blob
│   └── gcs.rs            # Google Cloud Storage
├── dto/
│   └── upload.response.rs
├── entities/
│   └── uploaded-file.rs
└── tests/
```

**Dependencies auto-added**: `uuid`

### Email Module

```bash
ironic generate ready-resource email
```

**Output** — creates `src/modules/email/` with:
```
email/
├── mod.rs
├── email.controller.rs  # Send email endpoint
├── email.service.rs     # Delivery logic
├── processors/
│   ├── mod.rs
│   ├── smtp.rs          # SMTP delivery
│   ├── ses.rs           # Amazon SES
│   ├── sendgrid.rs      # SendGrid API
│   ├── mailgun.rs       # Mailgun API
│   └── log.rs           # Development logger
├── dto/
│   └── send-email.dto.rs
├── entities/
│   └── email-message.rs
└── tests/
```

## Migration commands

Requires the `sqlx-postgres`, `sqlx-mysql`, or `sqlx-sqlite` feature for `up|down|status`.

| Command | What it does |
|---------|-------------|
| `ironic migrate create <name>` | Create a timestamped SQL migration file in `./migrations/` |
| `ironic migrate up` | Apply all pending migrations |
| `ironic migrate down --steps N` | Revert the last N migrations |
| `ironic migrate status` | Show applied vs pending migrations |

Ironic reads `DATABASE_URL` from the environment or `.env` file. Migration files follow the `sqlx` convention and are compatible with `sqlx-cli`.

```bash
# Typical workflow
ironic migrate create add_users_table
# edit migrations/1742169600_add_users_table.sql
ironic migrate up
ironic migrate status
```

For a full walkthrough, see [Database Migrations](/docs/data-auth/migrations).

## OpenAPI command

| Command | What it does |
|---------|-------------|
| `ironic openapi` | Auto-generate `openapi.json` — builds, starts, fetches spec, shuts down |
| `ironic openapi -p <name>` | Generate spec for a specific monorepo app |
| `ironic openapi -o spec.json --port 8081` | Custom output path and port |
| `ironic openapi --timeout 30` | Wait up to 30s for the service to start |

Requires the `openapi` feature in your `Cargo.toml`:

```toml
ironic = { workspace = true, features = ["openapi"] }
```

The command:
1. Builds the project (`cargo build`)
2. Starts the service on the specified port
3. Polls `http://localhost:{port}/openapi.json` until ready
4. Formats and writes the spec to the output file
5. Shuts the service down

### Generate YAML

Convert the JSON output to YAML with a tool like `yq`:

```bash
ironic openapi -o spec.json
yq -P eval spec.json > openapi.yaml
```

### Client SDK Generation

Use the generated spec to create typed clients:

```bash
# TypeScript
npx openapi-typescript spec.json -o client.ts

# Python
openapi-python-client generate --path spec.json
```

## Inspection commands

| Command | What it does |
|---------|-------------|
| `ironic routes` | List all routes in the project |
| `ironic graph` | Print a Graphviz dependency graph |

## Doctor command

```bash
ironic doctor
```

Checks your environment:

```
Rust                   OK rustc 1.0.0
Cargo                  OK cargo 1.0.0
Project manifest       OK /path/to/Cargo.toml
Ironic dependency      OK found
CLI version            OK 0.2.0 (latest)
```

## Update command

```bash
ironic update
# or: ironic upgrade
```

Checks crates.io for a newer version and shows update instructions.

## What you learned

- [x] `ironic new` creates projects
- [x] `ironic start/build/test` wraps Cargo commands
- [x] `ironic generate resource` creates full vertical slices
- [x] `ironic migrate create/up/down/status` manages database schema
- [x] `ironic doctor` diagnoses environment issues
- [x] `ironic routes` and `ironic graph` inspect projects
- [x] `ironic openapi` generates OpenAPI JSON specs
- [x] `ironic new --graphql` creates a GraphQL project
- [x] `ironic generate app --grpc` scaffolds gRPC microservices with DI
- [x] `ironic generate app --graphql` scaffolds GraphQL microservices with async-graphql
