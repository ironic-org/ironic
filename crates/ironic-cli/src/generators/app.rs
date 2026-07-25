use std::path::Path;

use crate::CliError;

use super::{GenerationReport, monorepo, naming, record, source};

/// Generates a new microservice app inside a monorepo workspace.
///
/// Creates a binary crate at `apps/<name>/` with:
/// - `Cargo.toml` — workspace-aware manifest with core dependencies
/// - `Dockerfile` — multi-stage production build
/// - `.env` — development environment variables
/// - `src/main.rs` — minimal async entry point with `AxumAdapter`
/// - `src/app.rs` — root `AppModule` definition
/// - `PRODUCTION.md` — production readiness checklist
///
/// If no `apps/` directory exists yet, auto-converts the current
/// single-service project to a monorepo workspace first.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, existing destinations, or filesystem errors.
pub fn generate_app(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();

    let apps_dir = root.join("apps");
    if !apps_dir.is_dir() {
        monorepo::convert_to_monorepo(root, &mut report)?;
    }

    let dest = root.join("apps").join(&names.raw);
    if dest.exists() {
        return Err(CliError::InvalidName {
            name: format!("app `{}` already exists at `{}`", names.raw, dest.display()),
        });
    }

    let port = next_port(root);
    let files = generate_app_files(&dest, &names, port);

    for (path, contents) in &files {
        write_app_file(path, contents, &mut report)?;
    }

    add_app_to_workspace(root, &names, &mut report);

    let dev_guide = format!(
        "run `cd apps/{} && cargo run` to start the service on port {}",
        names.raw, port
    );
    report.manual_instructions.push(dev_guide);

    Ok(report)
}

/// Returns the list of files to create for a new app, each as a `(path, content)` pair.
fn generate_app_files(
    dest: &Path,
    names: &naming::Names,
    port: u16,
) -> Vec<(std::path::PathBuf, String)> {
    vec![
        (dest.join("Cargo.toml"), app_manifest(names)),
        (dest.join("Dockerfile"), app_dockerfile(names, port)),
        (dest.join(".env"), app_env(names, port)),
        (dest.join("src/main.rs"), app_main(names, port)),
        (dest.join("src/app.rs"), app_module()),
        (dest.join("src/platform/mod.rs"), app_platform_mod()),
        (dest.join("src/platform/logging.rs"), app_platform_logging()),
        (dest.join("src/platform/config.rs"), app_platform_config()),
        (
            dest.join("PRODUCTION.md"),
            app_production_guide(&names.raw, port),
        ),
    ]
}

/// Creates parent directories and writes a generated file if content differs.
fn write_app_file(
    path: &Path,
    contents: &str,
    report: &mut GenerationReport,
) -> Result<(), CliError> {
    use std::fs;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CliError::Io {
            action: "create directory",
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    let changed = source::write_generated(path, contents)?;
    record(report, path, changed);
    Ok(())
}

/// Registers the new app as a workspace member in the root `Cargo.toml`.
fn add_app_to_workspace(root: &Path, names: &naming::Names, report: &mut GenerationReport) {
    let workspace_toml = root.join("Cargo.toml");
    if workspace_toml.is_file() {
        let contents = std::fs::read_to_string(&workspace_toml).unwrap_or_default();
        if !contents.contains(&format!("apps/{}", names.raw)) {
            monorepo::ensure_workspace_member(&workspace_toml, &format!("apps/{}", names.raw));
        }
    } else {
        report.manual_instructions.push(format!(
            "add `apps/{}` to your workspace members in Cargo.toml",
            names.raw
        ));
    }
}

/// Assigns the next available port (8080 + app count).
fn next_port(root: &Path) -> u16 {
    let apps_dir = root.join("apps");
    let count = std::fs::read_dir(&apps_dir).ok().map_or(0, |entries| {
        entries.filter_map(std::result::Result::ok).count()
    });
    8080 + u16::try_from(count).unwrap_or(0)
}

/// Returns the content for the app's `Cargo.toml`.
fn app_manifest(names: &naming::Names) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = {{ workspace = true }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
garde = {{ workspace = true }}
sqlx = {{ workspace = true }}
tracing = {{ workspace = true }}
tracing-subscriber = {{ workspace = true }}
dotenvy = {{ workspace = true }}

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
"#,
        name = names.raw
    )
}

/// Returns the content for the app's multi-stage `Dockerfile`.
///
/// Uses Alpine + musl for fully static binaries and a `scratch` final stage
/// for minimal production images (~10 MB).
fn app_dockerfile(names: &naming::Names, port: u16) -> String {
    let bin = &names.raw;
    format!(
        r#"FROM rust:1.97-alpine AS builder
WORKDIR /app

# musl + static deps for fully linked binaries
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && \
    rustup target add x86_64-unknown-linux-musl

# Cache dependencies first (dummy source, then real build)
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {{}}" > src/main.rs
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; true

# Real build with actual source
COPY src ./src
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/{bin} /{bin}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT={port}
EXPOSE {port}
CMD ["/{bin}"]
"#,
    )
}

/// Returns the content for the app's `.env` file with default dev values.
fn app_env(names: &naming::Names, port: u16) -> String {
    format!(
        r"SERVER_HOST=0.0.0.0
SERVER_PORT={port}
RUST_LOG=info
CORS_ORIGINS=[]
RATE_LIMIT_MAX=100
DATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/{name}
",
        port = port,
        name = names.raw
    )
}

#[allow(clippy::too_many_lines)]
pub(super) fn app_production_guide(name: &str, _port: u16) -> String {
    format!(
        r#"# Production Readiness Guide — {name}

This document outlines what you should add before deploying `{name}` to production.

## Middleware

Add these middleware layers in `src/main.rs` (in order):

```rust
use std::time::Duration;
use ironic::security::{{CorsConfig, CorsMiddleware, RateLimitMiddleware, SecurityHeadersConfig, SecurityHeadersMiddleware}};
use ironic::metrics::{{MetricsLayer, MetricsConfig}};

Application::builder()
    .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
    .middleware(RateLimitMiddleware::new(100, 60))  // 100 req/min per IP
    .middleware(CorsMiddleware::new(
        CorsConfig::new().allowed_origins(vec!["https://your-frontend.com"]),
    ))
```

| Middleware | Purpose | Crate Feature |
|---|---|---|
| `SecurityHeadersMiddleware` | Helmet-style headers (XSS, CSP, HSTS) | `security` |
| `RateLimitMiddleware` | Per-IP rate limiting | `security` |
| `CorsMiddleware` | CORS for frontend access | `security` |
| `MetricsLayer` | Prometheus metrics endpoint | `metrics` |
| Request body limit | Payload size cap | built-in |
| Request timeout | Drop slow requests | built-in |

Enable the `security` and `metrics` features in your `Cargo.toml`:

```toml
ironic = {{ workspace = true, features = ["security", "metrics", "logging", "openapi"] }}
```

## Platform Configuration

Add compression, timeouts, and OpenAPI to the `AxumAdapter`:

```rust
.platform(
    AxumAdapter::new()
        .compression()                                    // gzip/deflate
        .request_body_limit(5 * 1024 * 1024)              // 5 MB
        .request_timeout(Duration::from_secs(30))          // 30s timeout
        .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
        .with_openapi(OpenApiConfig::new("{name}", "0.1.0"))
        .swagger_ui("/docs"),
)
```

## OpenAPI / Swagger Docs

Once configured, the framework auto-generates an OpenAPI 3.1 JSON spec at runtime:

```
Service     Spec URL              Swagger UI
───────     ────────              ──────────
api-gateway http://localhost:8080/openapi.json  http://localhost:8080/docs
auth        http://localhost:8081/openapi.json  http://localhost:8081/docs
```

Each service serves its own spec independently because each is a separate binary
with its own route tree.

### Via the CLI (Recommended)

The `ironic openapi` command handles build, startup, fetch, and shutdown automatically:

```bash
# Generate openapi.json for the current service
ironic openapi

# For a specific app in a monorepo
ironic openapi -p api-gateway

# Custom port and output path
ironic openapi -p auth-service --port 8081 -o docs/openapi.json
```

### Via curl (Manual)

Useful for CI/CD pipelines:

```bash
curl http://localhost:8080/openapi.json > api-gateway-spec.json

# Validate with a linter
npx @redocly/cli lint api-gateway-spec.json
```

### Client SDK Generation

Use the exported JSON spec to generate typed clients:

```bash
# TypeScript / JavaScript
npx openapi-typescript openapi.json -o client.ts

# Python
openapi-python-client generate --path openapi.json

# Go
openapi-generator-cli generate -i openapi.json -g go -o ./client
```

Requires the `openapi` feature in your `Cargo.toml` to enable the spec endpoint:

```toml
ironic = {{ workspace = true, features = ["openapi"] }}
```

## Observability

Add tracing to see what your app is doing:

```rust
// src/main.rs — before building the application
tracing_subscriber::fmt()
    .with_env_filter("info,{name}=debug")
    .with_target(true)
    .with_file(true)
    .with_line_number(true)
    .compact()
    .init();
```

Enable structured JSON logging for production:

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter("info")
    .init();
```

## Error Handling

Create a global exception filter to catch unhandled errors:

```rust
struct GlobalExceptionFilter;

impl Filter for GlobalExceptionFilter {{
    fn catch(&self, exception: &dyn std::error::Error) -> Result<(), HttpError> {{
        tracing::error!(%exception, "unhandled exception");
        Err(HttpError::internal_server_error("INTERNAL_ERROR", "something went wrong"))
    }}
}}
```

Attach it via `Application::builder().filter(GlobalExceptionFilter)`.

## Database

Enable a database connection pool:

```rust
use std::sync::OnceLock;
use sqlx::postgres::PgPool;

static DB: OnceLock<PgPool> = OnceLock::new();

pub fn db() -> &'static PgPool {{
    DB.get().expect("database not initialized")
}}

pub async fn init_db(url: &str) -> PgPool {{
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!().run(&pool).await.expect("migrations failed");
    let _ = DB.set(pool.clone());
    pool
}}
```

## Health Checks

Add a health-check endpoint to your root module:

```rust
// src/app.rs
use ironic::health::HealthModule;

#[derive(Module)]
#[module(imports = [HealthModule], ...)]
pub struct AppModule;
```

This exposes `GET /health` returning `{{"status": "ok"}}`.

## Testing

```bash
# Unit tests
cargo test

# With logging
cargo test -- --nocapture

# Integration (requires database)
DATABASE_URL=postgres://... cargo test --test integration
```

## Graceful Shutdown

The framework handles SIGTERM/SIGINT automatically. Ensure your reverse proxy (nginx, Traefik, ALB) waits for the health check to fail before draining connections.

## Security Checklist

- [ ] CORS origins restricted to known frontends
- [ ] Rate limiting enabled (start with 100 req/min per IP)
- [ ] Security headers enabled (XSS, CSP, HSTS, frame-guard)
- [ ] Request body size limited (5 MB default)
- [ ] Request timeout set (30s default)
- [ ] `RUST_LOG` set to `info` or `warn` in production
- [ ] Database credentials use environment variables, not defaults
- [ ] OpenAPI docs disabled or behind auth in production

## Performance Checklist

- [ ] Release builds: `ironic build -- --release`
- [ ] Compression enabled via `.compression()` on the adapter
- [ ] Connection pooling tuned (start with 10, monitor)
- [ ] Metrics exported and scraped by Prometheus
- [ ] Structured JSON logging for log aggregation
"#,
    )
}

/// Returns the content for the app's entry point `src/main.rs`.
fn app_main(names: &naming::Names, port: u16) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"mod app;
mod platform;

use ironic::prelude::*;

use app::AppModule;

#[ironic::main]
async fn main() {{
    dotenvy::dotenv().ok();
    platform::logging::init();

    let addr = platform::config::listen_addr("{port}");
    let app = Application::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{addr}} (ironic v{version})");
    app.listen(&addr).await.expect("server failed");
}}
"#,
        name = names.raw,
        port = port
    )
}

/// Returns the content for the root module `src/app.rs`.
fn app_module() -> String {
    "use ironic::prelude::*;\n\n#[derive(Module)]\n#[module()]\npub struct AppModule;\n".to_string()
}

/// Returns the content for `src/platform/mod.rs`.
fn app_platform_mod() -> String {
    "pub mod config;\npub mod logging;\n".to_string()
}

/// Returns the content for `src/platform/logging.rs`.
fn app_platform_logging() -> String {
    r#"use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
"#
    .to_string()
}

/// Returns the content for `src/platform/config.rs`.
fn app_platform_config() -> String {
    r#"use std::env;

pub fn listen_addr(default_port: &str) -> String {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| default_port.into());
    format!("{host}:{port}")
}
"#
    .to_string()
}

/// Generates a reusable library crate with an Ironic module scaffold.
///
/// Creates a Cargo library at `libs/<name>/` (monorepo) or `<name>/` (standalone) with:
/// - `Cargo.toml` — workspace-aware library manifest with `#[lib]` section
/// - `src/lib.rs` — public module re-export
/// - `src/mod.rs` — `#[derive(Module)]` struct
///
/// Automatically registers the library as a workspace member when inside a monorepo.
///
/// # Errors
///
/// Returns [`CliError`] when the destination is occupied or files cannot be written.
pub fn generate_library(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();

    let is_workspace = root.join("apps").is_dir();
    let dest = if is_workspace {
        root.join("libs").join(&names.kebab)
    } else {
        root.join(&names.kebab)
    };

    if dest.exists() {
        return Err(CliError::InvalidName {
            name: format!("directory `{}` already exists", dest.display()),
        });
    }

    let files = [
        (dest.join("Cargo.toml"), library_manifest(&names.kebab)),
        (dest.join("src/lib.rs"), library_src_lib(&names)),
        (dest.join("src/mod.rs"), library_module_shell(&names)),
    ];

    for (path, contents) in &files {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CliError::Io {
                action: "create directory",
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let changed = source::write_generated(path, contents)?;
        record(&mut report, path, changed);
    }

    if is_workspace {
        let member = format!("libs/{}", names.kebab);
        let workspace_toml = root.join("Cargo.toml");
        if workspace_toml.is_file() {
            monorepo::ensure_workspace_member(&workspace_toml, &member);
        }
    }

    report.manual_instructions.push(format!(
        "add `{} = {{ path = \"{}\" }}` to your project's Cargo.toml dependencies",
        names.kebab,
        dest.display()
    ));

    Ok(report)
}

/// Returns the content for a library crate's `Cargo.toml`.
fn library_manifest(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
description = "An Ironic library crate"

[dependencies]
ironic = {{ workspace = true }}

[lib]
name = "{name}"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
"#
    )
}

/// Returns the content for a library crate's `src/lib.rs`.
fn library_src_lib(names: &naming::Names) -> String {
    format!(
        r"pub mod r#mod;

pub use r#mod::{name}Module;
",
        name = names.pascal
    )
}

/// Returns the content for a library crate's `src/mod.rs` defining the root module.
fn library_module_shell(names: &naming::Names) -> String {
    format!(
        r"use ::ironic::prelude::*;

#[derive(Module)]
#[module()]
pub struct {name}Module;
",
        name = names.pascal
    )
}
