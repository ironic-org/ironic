use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    app::{app_controller, app_production_guide, app_service},
    common::{naming::Names, source::write_generated},
};
use crate::CliError;

/// Result of creating a new project.
#[derive(Debug)]
pub struct ProjectReport {
    /// Created project directory.
    pub destination: PathBuf,
}

// ── Public API ─────────────────────────────────────────────────────────

/// Returns the normalized destination directory for a project name.
///
/// # Errors
///
/// Returns [`CliError`] when `name` contains no usable identifier characters.
pub fn directory_name(name: &str) -> Result<String, CliError> {
    Ok(Names::parse(name)?.kebab)
}

/// Derives a normalized project name from an existing directory.
///
/// # Errors
///
/// Returns [`CliError`] when the directory has no file name or its name cannot form a safe Rust
/// identifier.
pub fn name_from_directory(directory: &Path) -> Result<String, CliError> {
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::InvalidName {
            name: directory.display().to_string(),
        })?;
    directory_name(name)
}

/// Creates a complete application scaffold.
///
/// # Errors
///
/// Returns [`CliError`] when the destination is occupied or files cannot be created.
#[allow(clippy::too_many_lines)]
pub fn create(
    destination: &Path,
    name: &str,
    framework_workspace: Option<&Path>,
    graphql: bool,
) -> Result<ProjectReport, CliError> {
    let version = env!("CARGO_PKG_VERSION");
    let names = Names::parse(name)?;
    let manifest = manifest(&names.kebab, framework_workspace);

    let dep_spec = framework_workspace.map_or_else(
        || "version = \"1.1\"".to_string(),
        |w| format!("path = \"{}\", default-features = false", toml_path(w)),
    );
    let files: Vec<(std::path::PathBuf, String)> = if graphql {
        vec![
            (
                destination.join("Cargo.toml"),
                manifest_graphql_with_dep(&names.kebab, &dep_spec),
            ),
            (
                destination.join("ironic.toml"),
                project_config(&names.kebab),
            ),
            (
                destination.join(".env.example"),
                dotenv_example(&names.kebab),
            ),
            (destination.join(".gitignore"), gitignore().into()),
            (destination.join("Dockerfile"), dockerfile(&names.kebab)),
            (
                destination.join("rust-toolchain.toml"),
                rust_toolchain().into(),
            ),
            (destination.join("README.md"), readme(&names.kebab)),
            (
                destination.join("src/main.rs"),
                main_source_graphql(&names.kebab),
            ),
            (destination.join("src/app.rs"), app_source_graphql()),
            (
                destination.join("src/app_service.rs"),
                app_service_graphql(),
            ),
            (
                destination.join("PRODUCTION.md"),
                app_production_guide(&names.kebab, 8080),
            ),
            (destination.join("src/platform/mod.rs"), platform_mod()),
            (
                destination.join("src/platform/logging.rs"),
                platform_logging(),
            ),
            (
                destination.join("src/platform/config.rs"),
                platform_config(),
            ),
        ]
    } else {
        vec![
            (destination.join("Cargo.toml"), manifest),
            (
                destination.join("ironic.toml"),
                project_config(&names.kebab),
            ),
            (
                destination.join(".env.example"),
                dotenv_example(&names.kebab),
            ),
            (destination.join(".gitignore"), gitignore().into()),
            (destination.join("Dockerfile"), dockerfile(&names.kebab)),
            (
                destination.join("rust-toolchain.toml"),
                rust_toolchain().into(),
            ),
            (destination.join("README.md"), readme(&names.kebab)),
            (destination.join("src/main.rs"), main_source(&names.kebab)),
            (destination.join("src/app.rs"), app_source()),
            (
                destination.join("src/app_controller.rs"),
                app_controller().into(),
            ),
            (
                destination.join("src/app_service.rs"),
                app_service(&names.kebab, version),
            ),
            (
                destination.join("PRODUCTION.md"),
                app_production_guide(&names.kebab, 8080),
            ),
            (destination.join("src/platform/mod.rs"), platform_mod()),
            (
                destination.join("src/platform/logging.rs"),
                platform_logging(),
            ),
            (
                destination.join("src/platform/config.rs"),
                platform_config(),
            ),
        ]
    };

    // Validate all owned paths before writing. Allow pre-existing non-source files
    // (README.md, .gitignore, etc.) to be preserved; error on source file conflicts.
    let source_patterns = ["Cargo.toml", "ironic.toml", "src/"];
    for (path, contents) in &files {
        if path.exists() {
            let existing =
                fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?;
            let path_str = path.to_string_lossy();
            let is_source = source_patterns.iter().any(|p| path_str.contains(p));
            if is_source && existing != *contents {
                return Err(CliError::FileConflict {
                    path: path.to_owned(),
                });
            }
        }
    }

    fs::create_dir_all(destination)
        .map_err(|error| CliError::io("create directory", destination, error))?;
    for (path, contents) in files {
        if !path.exists() {
            write_generated(&path, &contents)?;
        }
    }
    Ok(ProjectReport {
        destination: destination.to_owned(),
    })
}

// ── Manifest ───────────────────────────────────────────────────────────

fn manifest(name: &str, workspace: Option<&Path>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let range = version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
    let dep_spec = workspace.map_or_else(
        || format!("version = \"{range}\""),
        |w| format!("path = \"{}\", default-features = false", toml_path(w)),
    );
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.97"
publish = false

[dependencies]
ironic = {{ features = ["security", "compression", "metrics", "validation", "versioning", "openapi", "logging", "sqlx-postgres"], {dep_spec} }}
"#,
    )
}

// ── Project scaffolding ────────────────────────────────────────────────

fn main_source(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"mod app;
mod app_controller;
mod app_service;
mod platform;

use ironic::prelude::*;

use app::AppModule;

#[ironic::main]
async fn main() {{
    ironic::dotenvy::dotenv().ok();
    platform::logging::init();

    let addr = platform::config::listen_addr("8080");
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{addr}} (ironic v{version})");
    app.listen(&addr).await.expect("server failed");
}}
"#,
    )
}

fn app_source() -> String {
    r"use ironic::prelude::*;
use crate::app_controller::AppController;
use crate::app_service::AppService;

#[derive(Module)]
#[module(
    controllers = [AppController],
    providers = [AppService],
)]
pub struct AppModule;
"
    .to_string()
}

fn manifest_graphql_with_dep(name: &str, dep_spec: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.97"
publish = false

[dependencies]
ironic = {{ features = ["graphql", "logging"], {dep_spec} }}
async-graphql = {{ version = "7", features = ["graphiql"] }}

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
"#,
    )
}

fn main_source_graphql(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"mod app;
mod app_service;
mod platform;

use std::sync::Arc;

use ironic::prelude::*;
use app::AppModule;
use app_service::AppService;

type GqlSchema = ironic::async_graphql::Schema<AppService, ironic::async_graphql::EmptyMutation, ironic::async_graphql::EmptySubscription>;

#[ironic::main]
async fn main() {{
    ironic::dotenvy::dotenv().ok();
    platform::logging::init();

    let schema = Arc::new(
        ironic::async_graphql::Schema::build(AppService, ironic::async_graphql::EmptyMutation, ironic::async_graphql::EmptySubscription)
            .data(AppModule)
            .finish()
    );

    async fn graphql_handler(
        schema: ironic::axum::Extension<Arc<GqlSchema>>,
        request: ironic::axum::Json<ironic::async_graphql::Request>,
    ) -> ironic::axum::Json<ironic::async_graphql::Response> {{
        let response = schema.execute(request.0).await;
        ironic::axum::Json(response)
    }}

    let addr = platform::config::listen_addr("8080");
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(
            AxumAdapter::new().configure_router(move |router| {{
                router
                    .route("/graphql", ironic::axum::routing::post(graphql_handler))
                    .layer(ironic::axum::Extension(schema))
            }})
        )
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{addr}}/graphql (ironic v{version})");
    app.listen(&addr).await.expect("server failed");
}}
"#,
    )
}

fn app_source_graphql() -> String {
    r"use ironic::prelude::*;
use crate::app_service::AppService;

#[derive(Module)]
#[module(providers = [AppService])]
pub struct AppModule;
"
    .to_string()
}

fn app_service_graphql() -> String {
    r#"use ironic::prelude::*;
use async_graphql::{Object, Context, Result};

#[derive(Injectable)]
pub struct AppService;

#[Object]
impl AppService {
    async fn hello(&self, _ctx: &Context<'_>) -> Result<String> {
        Ok("Hello from GraphQL!".into())
    }
}
"#
    .to_string()
}

fn project_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"
source_root = "src"
default_module = "src/app.rs"

[generate]
module_path = "src/modules"
"#,
    )
}

// ── Platform ───────────────────────────────────────────────────────────

fn platform_mod() -> String {
    "pub mod config;\npub mod logging;\n".to_string()
}

fn platform_logging() -> String {
    r#"
pub fn init() {
    let filter = ironic::tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    ironic::tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
"#
    .to_string()
}

fn platform_config() -> String {
    r#"use std::env;

pub fn listen_addr(default_port: &str) -> String {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| default_port.into());
    format!("{host}:{port}")
}
"#
    .to_string()
}

// ── Infrastructure ─────────────────────────────────────────────────────

fn dotenv_example(name: &str) -> String {
    format!(
        r#"# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Logging
RUST_LOG=info

# Security ──────────────────────────────────────────────
# Security headers (HSTS, CSP, X-Frame-Options, etc.) are always on with
# secure defaults. You can customise them in src/main.rs.
#
# JSON array of allowed origins; leave empty to deny all cross-origin requests
# Example: CORS_ORIGINS=["https://app.com","https://admin.com"]
CORS_ORIGINS=[]
# Maximum requests per IP per 60-second window
RATE_LIMIT_MAX=100

# Database
DATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/{name}

# Redis (uncomment to use)
# REDIS_URL=redis://localhost:6379
"#,
    )
}

fn gitignore() -> &'static str {
    "/target\n**/*.rs.bk\n.env\n*.log\n.DS_Store\n*.pdb\n"
}

fn dockerfile(name: &str) -> String {
    let binary = name.replace('-', "_");
    format!(
        r#"FROM rust:1.97-alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev pkgconfig && \
    rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {{}}" > src/main.rs
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; true

COPY src ./src
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/{binary} /{binary}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
EXPOSE 8080
CMD ["/{binary}"]
"#,
    )
}

fn rust_toolchain() -> &'static str {
    r#"[toolchain]
channel = "1.97"
components = ["rustfmt", "clippy"]
"#
}

fn readme(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r"# {name}

Built with [Ironic](https://github.com/ironic-org/ironic) v{version}.

## Quick start

```bash
# Run with hot reload
ironic dev

# Or run directly
ironic start
```

Open http://localhost:8080 in your browser.

## Commands

| Task | Command |
|------|--------|
| Dev server | `ironic dev` |
| Run | `ironic start` |
| Build | `ironic build` |
| Test | `ironic test` |
| OpenAPI spec | `ironic openapi` |

## Environment

Copy `.env.example` to `.env` and adjust values.
",
    )
}

// ── Helpers ────────────────────────────────────────────────────────────

fn toml_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}
