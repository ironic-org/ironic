mod file_upload_email;
mod naming;
/// New-project scaffolding.
pub mod project;
/// Production-ready resource generators (authentication, authorization, etc.).
pub mod ready_resource;
mod source;
mod templates;

/// Generates an email module with configurable delivery backends.
pub use file_upload_email::generate_ready_resource_email;
/// Generates a file upload module with configurable storage backends.
pub use file_upload_email::generate_ready_resource_file_upload;
/// Generates a full authentication module.
pub use ready_resource::generate_ready_resource;
/// Generates a GraphQL resolver scaffold.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_graphql_resolver(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();
    let path = root
        .join("src")
        .join(format!("{}_resolver.rs", names.snake));
    let contents = format!(
        r#"use ::ironic::prelude::*;

#[resolver]
pub struct {name}Resolver;

#[gql_query]
async fn {snake}_query(&self) -> String {{
    "Hello from {name}!".to_string()
}}
"#,
        name = names.pascal,
        snake = names.snake,
    );
    let changed = source::write_generated(&path, &contents)?;
    record(&mut report, &path, changed);
    Ok(report)
}

/// Generates a basic auth module (passwords + sessions).
pub use ready_resource::generate_ready_resource_basic;
/// Generates a JWT-only auth module.
pub use ready_resource::generate_ready_resource_jwt;
/// Generates an OAuth-only auth module.
pub use ready_resource::generate_ready_resource_oauth;

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Generates a new microservice app inside a monorepo workspace.
///
/// Creates a binary crate in `apps/<name>/` with Ironic framework scaffold.
/// Use this inside a workspace to add microservices to your monorepo.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_app(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();

    // Auto-convert single-service project to monorepo when first app is generated
    let apps_dir = root.join("apps");
    if !apps_dir.is_dir() {
        convert_to_monorepo(root, &mut report)?;
    }

    let dest = root.join("apps").join(&names.kebab);
    if dest.exists() {
        return Err(CliError::InvalidName {
            name: format!(
                "app `{}` already exists at `{}`",
                names.kebab,
                dest.display()
            ),
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
        names.kebab, port
    );
    report.manual_instructions.push(dev_guide);

    Ok(report)
}

fn generate_app_files(
    dest: &Path,
    names: &naming::Names,
    port: u16,
) -> Vec<(std::path::PathBuf, String)> {
    let example_mod = "src/modules/example";
    vec![
        (dest.join("Cargo.toml"), app_manifest(names)),
        (dest.join("Dockerfile"), app_dockerfile(names, port)),
        (dest.join(".env"), app_env(names, port)),
        (dest.join("src/main.rs"), app_main(names, port)),
        (dest.join("src/app.rs"), app_module(names)),
        (dest.join("src/welcome.rs"), app_welcome(names)),
        (dest.join("src/platform/mod.rs"), app_platform_mod().into()),
        (dest.join("src/platform/config.rs"), app_platform_config()),
        (
            dest.join("src/platform/telemetry.rs"),
            app_platform_telemetry(),
        ),
        (
            dest.join("src/platform/database.rs"),
            app_platform_database(),
        ),
        (
            dest.join("src/modules/mod.rs"),
            "pub mod example;\n".to_string(),
        ),
        (
            dest.join(format!("{example_mod}/mod.rs")),
            app_example_module(names),
        ),
        (
            dest.join(format!("{example_mod}/controller/mod.rs")),
            app_controller_mod(names),
        ),
        (
            dest.join(format!(
                "{example_mod}/controller/{}_controller.rs",
                names.snake
            )),
            app_example_controller(names),
        ),
        (
            dest.join(format!("{example_mod}/services/mod.rs")),
            app_services_mod(names),
        ),
        (
            dest.join(format!("{example_mod}/services/{}_service.rs", names.snake)),
            app_example_service(names),
        ),
        (
            dest.join(format!("{example_mod}/repositories/mod.rs")),
            app_repository_mod(names),
        ),
        (
            dest.join(format!(
                "{example_mod}/repositories/{}_repository.rs",
                names.snake
            )),
            app_example_repository(names),
        ),
        (
            dest.join(format!("{example_mod}/dto/mod.rs")),
            app_dto_mod(names),
        ),
        (
            dest.join(format!("{example_mod}/dto/create_{}_dto.rs", names.snake)),
            app_create_dto(names),
        ),
        (
            dest.join(format!("{example_mod}/dto/update_{}_dto.rs", names.snake)),
            app_update_dto(names),
        ),
        (
            dest.join(format!("{example_mod}/entities/mod.rs")),
            app_entities_mod(names),
        ),
        (
            dest.join(format!("{example_mod}/entities/{}.rs", names.snake)),
            app_entity(names),
        ),
        (
            dest.join(format!("{example_mod}/tests/mod.rs")),
            app_test_mod(),
        ),
        (
            dest.join(format!("{example_mod}/tests/unit.rs")),
            app_test_unit(names),
        ),
        (
            dest.join(format!("{example_mod}/tests/integration.rs")),
            app_test_integration(names),
        ),
    ]
}

fn write_app_file(
    path: &Path,
    contents: &str,
    report: &mut GenerationReport,
) -> Result<(), CliError> {
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

fn add_app_to_workspace(root: &Path, names: &naming::Names, report: &mut GenerationReport) {
    let workspace_toml = root.join("Cargo.toml");
    if workspace_toml.is_file() {
        let contents = std::fs::read_to_string(&workspace_toml).unwrap_or_default();
        if !contents.contains(&format!("apps/{}", names.kebab)) {
            ensure_workspace_member(&workspace_toml, &format!("apps/{}", names.kebab));
        }
    } else {
        report.manual_instructions.push(format!(
            "add `apps/{}` to your workspace members in Cargo.toml",
            names.kebab
        ));
    }
}

fn next_port(root: &Path) -> u16 {
    let apps_dir = root.join("apps");
    let count = std::fs::read_dir(&apps_dir).ok().map_or(0, |entries| {
        entries.filter_map(std::result::Result::ok).count()
    });
    8080 + u16::try_from(count).unwrap_or(0)
}

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
"#,
        name = names.kebab
    )
}

fn app_dockerfile(names: &naming::Names, port: u16) -> String {
    format!(
        r#"FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/{name} /app/{name}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT={port}
EXPOSE {port}
CMD ["./{name}"]
"#,
        name = names.kebab,
        port = port
    )
}

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
        name = names.kebab
    )
}

fn app_main(names: &naming::Names, port: u16) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"mod app;
mod modules;
mod platform;
mod welcome;

use std::time::Duration;

use ironic::{{AxumAdapter, OpenApiConfig, OpenApiAxumExt}};
use ironic::metrics::{{MetricsLayer, MetricsConfig}};
use ironic::prelude::*;
use ironic::security::{{
    CorsConfig, CorsMiddleware,
    RateLimitMiddleware,
    SecurityHeadersConfig, SecurityHeadersMiddleware,
}};

use app::AppModule;

struct GlobalExceptionMiddleware;

impl ironic::Middleware for GlobalExceptionMiddleware {{
    fn handle<'a>(
        &'a self,
        context: &'a mut ironic::RequestContext,
        next: ironic::MiddlewareNext<'a>,
    ) -> ironic::PipelineFuture<'a> {{
        Box::pin(async move {{
            match next.run(context).await {{
                Ok(response) => Ok(response),
                Err(error) => {{
                    let body = ironic::json::json!({{
                        "error": error.code(),
                        "message": error.message(),
                        "status": error.status().as_u16(),
                    }});
                    ironic::Response::json(error.status(), &body)
                }}
            }}
        }})
    }}
}}

#[ironic::main]
async fn main() {{
    dotenvy::dotenv().ok();
    platform::telemetry::init_tracing();

    let addr = format!(
        "{{}}:{{}}",
        platform::config::env("SERVER_HOST").unwrap_or_else(|| "0.0.0.0".into()),
        platform::config::env("SERVER_PORT").unwrap_or_else(|| "{port}".into()),
    );
    let cors_origins = platform::config::env_json_array("CORS_ORIGINS");
    let rate_limit_max: u64 = platform::config::env_parsed("RATE_LIMIT_MAX", 100u64);

    let application = Application::builder()
        .module(AppModule::definition())
        .middleware(GlobalExceptionMiddleware)
        .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
        .middleware(RateLimitMiddleware::new(rate_limit_max, 60))
        .middleware(CorsMiddleware::new(CorsConfig::new().allowed_origins(cors_origins)))
        .platform(
            AxumAdapter::new()
                .compression()
                .request_body_limit(5 * 1024 * 1024)
                .request_timeout(Duration::from_secs(30))
                .configure_router(|r| {{
                    r.layer(MetricsLayer::new(MetricsConfig::default()))
                }})
                .with_openapi(OpenApiConfig::new("{name}", "0.1.0"))
                .swagger_ui("/docs"),
        )
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{}} (ironic v{version})", addr);

    application
        .listen(&addr)
        .await
        .expect("application server failed");
}}
"#,
        name = names.kebab,
        port = port
    )
}

fn app_module(names: &naming::Names) -> String {
    format!(
        r"use ironic::prelude::*;
use crate::welcome::WelcomeModule;
use crate::modules::example::{}Module;
use ironic::metrics::MetricsModule;

#[derive(Module)]
#[module(
    imports = [HealthModule,
    MetricsModule,
    WelcomeModule,
    {}Module],
    providers = [],
    controllers = [],
    exports = [],
)]
pub struct AppModule;
",
        names.pascal, names.pascal
    )
}

fn app_welcome(names: &naming::Names) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"use ironic::prelude::*;

#[controller("/")]
#[derive(Injectable)]
struct WelcomeController;

#[routes]
impl WelcomeController {{
    #[get]
    async fn index(&self) -> Result<Json<serde_json::Value>, HttpError> {{
        Ok(Json(serde_json::json!({{
            "name": "{name}",
            "framework": "Ironic",
            "version": "{version}",
            "status": "running",
            "health": "/health",
            "docs": "/docs"
        }})))
    }}
}}

#[derive(Module)]
#[module(controllers = [WelcomeController])]
pub struct WelcomeModule;
"#,
        name = names.kebab
    )
}

fn app_platform_mod() -> &'static str {
    "pub mod config;\npub mod telemetry;\n// pub mod database;\n"
}

fn app_platform_config() -> String {
    r#"use std::env;

pub fn env(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn env_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn env_json_array(key: &str) -> Vec<String> {
    env::var(key)
        .ok()
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn server_address() -> String {
    let host = env("SERVER_HOST").unwrap_or_else(|| "0.0.0.0".into());
    let port = env("SERVER_PORT").unwrap_or_else(|| "8080".into());
    format!("{host}:{port}")
}
"#
    .to_string()
}

fn app_platform_telemetry() -> String {
    r#"use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();
}
"#
    .to_string()
}

fn app_platform_database() -> String {
    r#"//! Database connection pool (PostgreSQL via SQLx).
//!
//! # Setup
//!
//! 1. Set `DATABASE_URL` in your `.env` file:
//!
//!    ```env
//!    DATABASE_URL=postgres://user:password@localhost:5432/my_app
//!    ```
//!
//! 2. Uncomment `pub mod database;` in `src/platform/mod.rs`.
//!
//! 3. Initialize the pool at application startup (e.g. in `main.rs`):
//!
//!    ```rust
//!    use platform::database::build_pool;
//!    let pool = build_pool().await;
//!    ```
//!
//! 4. Access the pool anywhere in your app:
//!
//!    ```rust
//!    use platform::database::db;
//!    let row = sqlx::query("SELECT ..").fetch_one(db()).await?;
//!    ```
//!
//! # Migrations
//!
//! Create a `migrations/` directory with SQL migration files named using the
//! standard SQLx convention: `YYYYMMDD_HHMMSS_description.sql`.
//! Migrations are run automatically when `build_pool()` is called.

use std::sync::OnceLock;

pub static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

pub fn db() -> &'static sqlx::PgPool {
    DB_POOL
        .get()
        .expect("DATABASE_URL must be set and pool initialized")
}

#[allow(dead_code)]
pub async fn build_pool() -> sqlx::PgPool {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(super::config::env("DB_POOL_SIZE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10))
        .connect(&url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("invalid migrations directory")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    tracing::info!("database pool ready (max: {})", pool.size());
    pool
}
"#
    .to_string()
}

fn app_example_module(names: &naming::Names) -> String {
    format!(
        r"use ironic::prelude::*;

pub mod controller;
pub mod repositories;
pub mod services;
pub mod dto;
pub mod entities;

#[cfg(test)]
mod tests;

pub use controller::{}Controller;
pub use repositories::{}Repository;
pub use services::{}Service;

#[derive(Module)]
#[module(providers = [{}Repository, {}Service], controllers = [{}Controller])]
pub struct {}Module;
",
        names.pascal,
        names.pascal,
        names.pascal,
        names.pascal,
        names.pascal,
        names.pascal,
        names.pascal
    )
}

fn app_controller_mod(names: &naming::Names) -> String {
    format!(
        "pub mod {0}_controller;\npub use {0}_controller::{1}Controller;\n",
        names.snake, names.pascal
    )
}

fn app_example_controller(names: &naming::Names) -> String {
    format!(
        r#"use std::sync::Arc;
use ironic::prelude::*;
use super::super::services::{0}Service;
use crate::modules::example::dto::{{Create{0}Dto, Update{0}Dto}};
use crate::modules::example::entities::{0};

#[controller("/{1}")]
#[derive(Injectable)]
pub struct {0}Controller {{ service: Arc<{0}Service> }}

#[routes]
impl {0}Controller {{
    #[get]
    #[api(summary = "List all {1}", tag = "{0}", security = "bearer")]
    #[resp(200, "A list of {1}", json = Vec<{0}>)]
    async fn list(&self) -> Result<Json<Vec<{0}>>, HttpError> {{
        Ok(Json(self.service.list()))
    }}

    #[get("/:id")]
    #[api(summary = "Get a {1} by ID", tag = "{0}")]
    #[resp(200, "The requested {1}", json = {0})]
    #[resp(404, "{0} not found")]
    async fn get(&self, #[param] id: u64) -> Result<Json<{0}>, HttpError> {{
        self.service.find(id).map(Json)
    }}

    #[post]
    #[api(summary = "Create a new {1}", tag = "{0}")]
    #[body(json = Create{0}Dto)]
    #[resp(201, "{0} created", json = {0})]
    #[resp(400, "Validation error")]
    async fn create(&self, #[body] dto: Create{0}Dto) -> Result<Json<{0}>, HttpError> {{
        Ok(Json(self.service.create(dto)))
    }}

    #[put("/:id")]
    #[api(summary = "Update an existing {1}", tag = "{0}")]
    #[body(json = Update{0}Dto)]
    #[resp(200, "{0} updated", json = {0})]
    #[resp(404, "{0} not found")]
    async fn update(&self, #[param] id: u64, #[body] dto: Update{0}Dto) -> Result<Json<{0}>, HttpError> {{
        self.service.update(id, dto).map(Json)
    }}

    #[delete("/:id")]
    #[api(summary = "Delete a {1}", tag = "{0}")]
    #[resp(204, "{0} deleted")]
    #[resp(404, "{0} not found")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {{
        self.service.delete(id)
    }}
}}
"#,
        names.pascal, names.kebab
    )
}

fn app_services_mod(names: &naming::Names) -> String {
    format!(
        "pub mod {0}_service;\npub use {0}_service::{1}Service;\n",
        names.snake, names.pascal
    )
}

fn app_example_service(names: &naming::Names) -> String {
    format!(
        r"use std::sync::Arc;
use ironic::prelude::*;
use crate::modules::example::dto::{{Create{0}Dto, Update{0}Dto}};
use crate::modules::example::entities::{0};
use crate::modules::example::repositories::{0}Repository;

#[derive(Injectable)]
pub struct {0}Service {{
    pub repository: Arc<{0}Repository>,
}}

impl {0}Service {{
    pub fn list(&self) -> Vec<{0}> {{
        self.repository.list()
    }}

    pub fn find(&self, id: u64) -> Result<{0}, HttpError> {{
        self.repository.find(id)
    }}

    pub fn create(&self, dto: Create{0}Dto) -> {0} {{
        self.repository.create(dto.name, dto.description)
    }}

    pub fn update(&self, id: u64, dto: Update{0}Dto) -> Result<{0}, HttpError> {{
        self.repository.update(id, dto.name, dto.description)
    }}

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {{
        self.repository.delete(id)
    }}
}}
",
        names.pascal
    )
}

fn app_repository_mod(names: &naming::Names) -> String {
    format!(
        "pub mod {0}_repository;\npub use {0}_repository::{1}Repository;\n",
        names.snake, names.pascal
    )
}

fn app_example_repository(names: &naming::Names) -> String {
    format!(
        r#"use std::collections::HashMap;
use std::sync::Mutex;
use ironic::prelude::*;
use crate::modules::example::entities::{0};

static STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| Mutex::new(Store {{ items: HashMap::new(), next_id: 1 }}));

struct Store {{ items: HashMap<u64, {0}>, next_id: u64 }}

#[derive(Injectable)]
pub struct {0}Repository;

impl {0}Repository {{
    pub fn list(&self) -> Vec<{0}> {{
        STORE.lock().unwrap().items.values().cloned().collect()
    }}

    pub fn find(&self, id: u64) -> Result<{0}, HttpError> {{
        STORE.lock().unwrap().items.get(&id).cloned()
            .ok_or_else(|| HttpError::not_found("{0}_NOT_FOUND", format!("Item {{id}} not found")))
    }}

    pub fn create(&self, name: String, description: Option<String>) -> {0} {{
        let mut store = STORE.lock().unwrap();
        let id = store.next_id;
        store.next_id += 1;
        let item = {0} {{ id, name, description: description.unwrap_or_default() }};
        store.items.insert(id, item.clone());
        item
    }}

    pub fn update(&self, id: u64, name: Option<String>, description: Option<String>) -> Result<{0}, HttpError> {{
        let mut store = STORE.lock().unwrap();
        let item = store.items.get_mut(&id)
            .ok_or_else(|| HttpError::not_found("{0}_NOT_FOUND", format!("Item {{id}} not found")))?;
        if let Some(name) = name {{ item.name = name; }}
        if let Some(desc) = description {{ item.description = desc; }}
        Ok(item.clone())
    }}

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {{
        STORE.lock().unwrap().items.remove(&id)
            .map(|_| ())
            .ok_or_else(|| HttpError::not_found("{0}_NOT_FOUND", format!("Item {{id}} not found")))
    }}
}}
"#,
        names.pascal
    )
}

fn app_dto_mod(names: &naming::Names) -> String {
    format!(
        r"pub mod create_{0}_dto;
pub mod update_{0}_dto;
pub use create_{0}_dto::Create{1}Dto;
pub use update_{0}_dto::Update{1}Dto;
",
        names.snake, names.pascal
    )
}

fn app_create_dto(names: &naming::Names) -> String {
    format!(
        r"use garde::Validate;
use ironic::OpenApiSchema;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, OpenApiSchema)]
pub struct Create{0}Dto {{
    #[garde(length(min = 1, max = 256))]
    /// Item name (1–256 characters).
    pub name: String,
    #[garde(skip)]
    /// Optional description.
    pub description: Option<String>,
}}
",
        names.pascal
    )
}

fn app_update_dto(names: &naming::Names) -> String {
    format!(
        r"use ironic::OpenApiSchema;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]
pub struct Update{0}Dto {{
    /// New name (leave `null` to keep unchanged).
    pub name: Option<String>,
    /// New description (leave `null` to keep unchanged).
    pub description: Option<String>,
}}
",
        names.pascal
    )
}

fn app_entities_mod(names: &naming::Names) -> String {
    format!(
        "pub mod {0};\npub use {0}::{1};\n",
        names.snake, names.pascal
    )
}

fn app_entity(names: &naming::Names) -> String {
    format!(
        r"use ironic::OpenApiSchema;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]
pub struct {0} {{
    /// Unique identifier.
    pub id: u64,
    /// Item name.
    pub name: String,
    /// Item description.
    pub description: String,
}}
",
        names.pascal
    )
}

fn app_test_mod() -> String {
    r"/// Unit tests — service and business logic in isolation (no HTTP).
#[cfg(test)]
mod unit;
/// Integration tests — full HTTP request/response through the framework.
#[cfg(test)]
mod integration;
"
    .to_string()
}

fn app_test_unit(names: &naming::Names) -> String {
    format!(
        r#"//! Unit tests for `{}Service`.

use std::sync::Arc;
use crate::modules::example::dto::{{Create{0}Dto, Update{0}Dto}};
use crate::modules::example::repositories::{0}Repository;
use crate::modules::example::services::{0}Service;

fn service() -> {0}Service {{
    {0}Service {{ repository: Arc::new({0}Repository) }}
}}

#[test]
fn create_and_find() {{
    let svc = service();
    let item = svc.create(Create{0}Dto {{ name: "Test".into(), description: None }});
    assert_eq!(item.name, "Test");
    let found = svc.find(item.id).unwrap();
    assert_eq!(found.name, "Test");
}}

#[test]
fn update_works() {{
    let svc = service();
    let item = svc.create(Create{0}Dto {{ name: "Old".into(), description: None }});
    let updated = svc.update(item.id, Update{0}Dto {{ name: Some("New".into()), description: None }}).unwrap();
    assert_eq!(updated.name, "New");
}}

#[test]
fn delete_works() {{
    let svc = service();
    let item = svc.create(Create{0}Dto {{ name: "Del".into(), description: None }});
    assert!(svc.delete(item.id).is_ok());
    assert!(svc.find(item.id).is_err());
}}

#[test]
fn not_found_error() {{
    let svc = service();
    let err = svc.find(999).unwrap_err();
    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);
}}

#[test]
fn list_works() {{
    let svc = service();
    svc.create(Create{0}Dto {{ name: "A".into(), description: None }});
    svc.create(Create{0}Dto {{ name: "B".into(), description: None }});
    assert!(svc.list().len() >= 2);
}}
"#,
        names.pascal
    )
}

fn app_test_integration(names: &naming::Names) -> String {
    format!(
        r#"//! Integration tests for {0} — full HTTP request/response cycles.

use ironic::{{HttpStatus, TestApplication}};
use serde_json::json;

use super::super::*;

async fn app() -> TestApplication {{
    TestApplication::new::<{0}Module>().await.expect("test app must initialise")
}}

#[ironic::test]
async fn list_returns_ok() {{
    let a = app().await;
    assert_eq!(a.get("/{1}").send().await.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}}

#[ironic::test]
async fn create_and_get() {{
    let a = app().await;
    let resp = a.post("/{1}").json(&json!({{"name": "Test", "description": null}})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let id = resp.json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    assert_eq!(a.get(&format!("/{1}/{{id}}")).send().await.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}}

#[ironic::test]
async fn update_works() {{
    let a = app().await;
    let id = a.post("/{1}").json(&json!({{"name": "Old"}})).send().await
        .json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    let resp = a.put(&format!("/{1}/{{id}}")).json(&json!({{"name": "New"}})).send().await;
    assert_eq!(resp.json::<serde_json::Value>().unwrap()["name"], "New");
    a.shutdown().await.unwrap();
}}

#[ironic::test]
async fn delete_works() {{
    let a = app().await;
    let id = a.post("/{1}").json(&json!({{"name": "Del"}})).send().await
        .json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    a.delete(&format!("/{1}/{{id}}")).send().await;
    assert_eq!(a.get(&format!("/{1}/{{id}}")).send().await.status(), HttpStatus::NOT_FOUND);
    a.shutdown().await.unwrap();
}}

#[ironic::test]
async fn not_found_returns_404() {{
    let a = app().await;
    a.get("/{1}/999").send().await.assert_status(404);
    a.shutdown().await.unwrap();
}}
"#,
        names.pascal, names.kebab
    )
}

/// Converts a single-service project to a monorepo workspace.
/// Moves src/ into apps/<name>/, creates workspace Cargo.toml, sets up libs/.
/// Ensures Rust Analyzer can detect the workspace structure.
fn convert_to_monorepo(root: &Path, report: &mut GenerationReport) -> Result<(), CliError> {
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        return Err(CliError::InvalidName {
            name: "Cargo.toml not found — are you in an Ironic project?".into(),
        });
    }

    // Read current package name from Cargo.toml
    let toml_content = std::fs::read_to_string(&cargo_toml)
        .map_err(|e| CliError::io("read Cargo.toml", &cargo_toml, e))?;
    let pkg_name = toml_content
        .lines()
        .find_map(|l| l.strip_prefix("name = \""))
        .and_then(|l| l.split('"').next())
        .unwrap_or("app")
        .to_string();

    let app_dir = root.join("apps").join(&pkg_name);
    let src_dir = root.join("src");

    // Move src/ into apps/<name>/src/
    if src_dir.is_dir() {
        let dst_src = app_dir.join("src");
        fs::create_dir_all(&dst_src).map_err(|e| CliError::Io {
            action: "create apps directory",
            path: app_dir.clone(),
            source: e,
        })?;
        copy_dir_recursive(&src_dir, &dst_src)?;
        fs::remove_dir_all(&src_dir).map_err(|e| CliError::Io {
            action: "remove old src directory",
            path: src_dir,
            source: e,
        })?;
    }

    // Create app Cargo.toml (binary-only — no [lib] section)
    // Modules are declared directly in main.rs (mod app; mod modules; etc.)
    let app_manifest = format!(
        r#"[package]
name = "{pkg_name}"
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
"#
    );
    std::fs::write(app_dir.join("Cargo.toml"), &app_manifest).map_err(|e| CliError::Io {
        action: "write app Cargo.toml",
        path: app_dir.join("Cargo.toml"),
        source: e,
    })?;

    let ironic_version = env!("CARGO_PKG_VERSION");
    let version_range = ironic_version
        .splitn(3, '.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    // Rewrite root Cargo.toml as pure workspace manifest (no [package])
    let workspace_manifest = format!(
        r#"[workspace]
resolver = "3"
members = [
    "apps/{pkg_name}",
]

[workspace.dependencies]
ironic = {{ version = "{version_range}", features = ["security", "compression", "metrics", "validation", "versioning", "openapi", "logging", "sqlx-postgres"] }}
tokio = {{ version = "1", features = ["macros", "rt-multi-thread", "net", "signal"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
garde = "0.23"
sqlx = {{ version = "0.9", features = ["runtime-tokio", "postgres"] }}
tracing = {{ version = "0.1", features = ["attributes"] }}
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"
"#,
    );
    std::fs::write(&cargo_toml, &workspace_manifest).map_err(|e| CliError::Io {
        action: "write workspace Cargo.toml",
        path: cargo_toml.clone(),
        source: e,
    })?;

    create_monorepo_libs(root)?;

    report
        .manual_instructions
        .push("converted to monorepo — run `cargo check` to refresh Rust Analyzer".into());

    Ok(())
}

/// Creates a shared protobuf library directory in a monorepo workspace.
fn create_monorepo_libs(root: &Path) -> Result<(), CliError> {
    let lib_dir = root.join("libs").join("proto");
    let lib_dir_src = lib_dir.join("src");
    fs::create_dir_all(&lib_dir_src).map_err(|e| CliError::Io {
        action: "create lib directory",
        path: lib_dir.clone(),
        source: e,
    })?;
    let lib_cargo = r#"[package]
name = "proto"
version = "0.1.0"
edition = "2024"
"#
    .to_string();
    std::fs::write(lib_dir.join("Cargo.toml"), &lib_cargo).map_err(|e| CliError::Io {
        action: "write lib Cargo.toml",
        path: lib_dir.join("Cargo.toml"),
        source: e,
    })?;
    std::fs::write(lib_dir_src.join("lib.rs"), b"// shared library\n").map_err(|e| {
        CliError::Io {
            action: "write lib src",
            path: lib_dir_src.join("lib.rs"),
            source: e,
        }
    })?;
    Ok(())
}

/// Recursively copies a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), CliError> {
    for entry in std::fs::read_dir(src).map_err(|e| CliError::Io {
        action: "read source directory",
        path: src.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| CliError::Io {
            action: "read directory entry",
            path: src.to_path_buf(),
            source: e,
        })?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|e| CliError::Io {
            action: "get file type",
            path: entry_path.clone(),
            source: e,
        })?;
        let relative = entry_path
            .strip_prefix(src)
            .map_err(|_| CliError::InvalidName {
                name: "path error during copy".into(),
            })?;
        let target = dst.join(relative);
        if file_type.is_dir() {
            fs::create_dir_all(&target).map_err(|e| CliError::Io {
                action: "create directory",
                path: target.clone(),
                source: e,
            })?;
            copy_dir_recursive(&entry_path, &target)?;
        } else {
            fs::copy(&entry_path, &target).map_err(|e| CliError::Io {
                action: "copy file",
                path: entry_path,
                source: e,
            })?;
        }
    }
    Ok(())
}

fn ensure_workspace_member(manifest: &Path, member: &str) {
    use std::io::{Read, Write};
    let mut contents = String::new();
    if let Ok(mut f) = std::fs::File::open(manifest) {
        let _ = f.read_to_string(&mut contents);
    }
    let line = format!("        \"{member}\",\n");
    if let Some(pos) = contents.find("members = [") {
        let insert_pos = contents[pos..]
            .find(']')
            .map_or(contents.len(), |p| pos + p);
        contents.insert_str(insert_pos, &line);
        if let Ok(mut f) = std::fs::File::create(manifest) {
            let _ = f.write_all(contents.as_bytes());
        }
    }
}

/// Generates a reusable library crate.
///
/// Creates a Cargo library project with Ironic module scaffold.
/// If an `apps/` directory exists (monorepo), places it in `libs/<name>/`.
/// Otherwise, creates it as a standalone project in the current directory.
///
/// # Errors
///
/// Returns [`CliError`] when the destination is occupied or files cannot be written.
pub fn generate_library(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = naming::Names::parse(name)?;
    let mut report = GenerationReport::default();

    // Detect monorepo workspace (has apps/ directory)
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
            fs::create_dir_all(parent).map_err(|e| CliError::Io {
                action: "create directory",
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let changed = write_generated(path, contents)?;
        record(&mut report, path, changed);
    }

    if is_workspace {
        let member = format!("libs/{}", names.kebab);
        let workspace_toml = root.join("Cargo.toml");
        if workspace_toml.is_file() {
            ensure_workspace_member(&workspace_toml, &member);
        }
    }

    report.manual_instructions.push(format!(
        "add `{} = {{ path = \"{}\" }}` to your project's Cargo.toml dependencies",
        names.kebab,
        dest.display()
    ));

    Ok(report)
}

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
"#
    )
}

fn library_src_lib(names: &naming::Names) -> String {
    format!(
        r"pub mod r#mod;

pub use r#mod::{name}Module;
",
        name = names.pascal
    )
}

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

use naming::Names;
use source::{
    ensure_items, ensure_module_array_item, ensure_module_import, write_generated,
    write_module_shell,
};

use crate::CliError;

/// Files changed by a generator and any required manual follow-up.
#[derive(Debug, Default)]
pub struct GenerationReport {
    /// Newly created or safely updated files.
    pub created: Vec<PathBuf>,
    /// Existing files that already matched the deterministic output.
    pub unchanged: Vec<PathBuf>,
    /// Source registrations that require a human decision.
    pub manual_instructions: Vec<String>,
}

/// Generates an application module.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicting files, or unsafe source edits.
pub fn generate_module(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let mut report = GenerationReport::default();
    record(
        &mut report,
        &module_dir.join("mod.rs"),
        write_module_shell(&module_dir.join("mod.rs"), &names.pascal)?,
    );
    register_root_module(root, &names, &mut report)?;
    ensure_main_registration(root, &mut report);
    ensure_app_import(root, &names, &mut report);
    Ok(report)
}

/// Generates a controller inside a same-named module.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicts, or unsafe owned-module edits.
pub fn generate_controller(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let mut report = generate_module(root, name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let controller_dir = module_dir.join("controller");
    let file_name = format!("{}_controller.rs", names.snake);
    let path = controller_dir.join(&file_name);
    record(
        &mut report,
        &path,
        write_generated(&path, &templates::controller(&names))?,
    );
    write_generated(
        &controller_dir.join("mod.rs"),
        &templates::controller_mod(&names),
    )?;
    ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod controller;",
            &format!("pub use controller::{}Controller;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let controller_type = format!("{}Controller", names.pascal);
        if let Err(error) = ensure_module_array_item(&module_mod, &controller_type, "controllers") {
            report.manual_instructions.push(format!(
                "add `{controller_type}` to `controllers = [...]` on `{}Module` ({error})",
                names.pascal
            ));
        }
    }
    Ok(report)
}

/// Generates a repository inside a same-named module.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicts, or unsafe owned-module edits.
pub fn generate_repository(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let mut report = generate_module(root, name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let repos_dir = module_dir.join("repositories");
    let path = repos_dir.join(format!("{}_repository.rs", names.snake));
    record(
        &mut report,
        &path,
        write_generated(&path, &templates::repository(&names))?,
    );
    write_generated(
        &repos_dir.join("mod.rs"),
        &templates::repository_mod(&names),
    )?;
    ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod repositories;",
            &format!("pub use repositories::{}Repository;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let repo_type = format!("{}Repository", names.pascal);
        if let Err(error) = ensure_module_array_item(&module_mod, &repo_type, "providers") {
            report.manual_instructions.push(format!(
                "add `{repo_type}` to `providers = [...]` on `{}Module` ({error})",
                names.pascal
            ));
        }
    }
    Ok(report)
}

/// Generates a dependency-injectable service inside a same-named module.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicts, or unsafe owned-module edits.
pub fn generate_service(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let mut report = generate_module(root, name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let services_dir = module_dir.join("services");
    let path = services_dir.join(format!("{}_service.rs", names.snake));
    record(
        &mut report,
        &path,
        write_generated(&path, &templates::service(&names))?,
    );
    write_generated(
        &services_dir.join("mod.rs"),
        &templates::services_mod(&names),
    )?;
    ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod services;",
            &format!("pub use services::{}Service;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let service_type = format!("{}Service", names.pascal);
        if let Err(error) = ensure_module_array_item(&module_mod, &service_type, "providers") {
            report.manual_instructions.push(format!(
                "add `{service_type}` to `providers = [...]` on `{}Module` ({error})",
                names.pascal
            ));
        }
    }
    Ok(report)
}

/// Generates a complete module, service, and controller vertical slice.
///
/// Creates the following structure inside `src/modules/{name}/`:
///
/// ```text
/// mod.rs
/// tests/
///   mod.rs             — test entry (declares unit + integration)
///   unit.rs            — business logic tests (no HTTP)
///   integration.rs     — full HTTP request/response tests
/// controller/
///   mod.rs
///   {name}_controller.rs
/// repositories/
///   mod.rs
///   {name}_repository.rs
/// services/
///   mod.rs
///   {name}_service.rs
/// dto/
///   mod.rs
///   create_{name}_dto.rs
///   update_{name}_dto.rs
/// entities/
///   mod.rs
///   {name}.rs
/// ```
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicting files, or unsafe source edits.
pub fn generate_resource(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let controller_dir = module_dir.join("controller");
    let repositories_dir = module_dir.join("repositories");
    let services_dir = module_dir.join("services");
    let dto_dir = module_dir.join("dto");
    let entities_dir = module_dir.join("entities");
    let tests_dir = module_dir.join("tests");
    let mut report = GenerationReport::default();
    let files = [
        (
            module_dir.join("mod.rs"),
            templates::resource_module(&names),
        ),
        (tests_dir.join("mod.rs"), templates::test_mod(&names)),
        (tests_dir.join("unit.rs"), templates::test_unit(&names)),
        (
            tests_dir.join("integration.rs"),
            templates::test_integration(&names),
        ),
        (
            controller_dir.join("mod.rs"),
            templates::controller_mod(&names),
        ),
        (
            controller_dir.join(format!("{}_controller.rs", names.snake)),
            templates::resource_controller(&names),
        ),
        (
            repositories_dir.join("mod.rs"),
            templates::repository_mod(&names),
        ),
        (
            repositories_dir.join(format!("{}_repository.rs", names.snake)),
            templates::repository(&names),
        ),
        (services_dir.join("mod.rs"), templates::services_mod(&names)),
        (
            services_dir.join(format!("{}_service.rs", names.snake)),
            templates::service(&names),
        ),
        (dto_dir.join("mod.rs"), templates::dto_mod(&names)),
        (
            dto_dir.join(format!("create_{}_dto.rs", names.snake)),
            templates::create_dto(&names),
        ),
        (
            dto_dir.join(format!("update_{}_dto.rs", names.snake)),
            templates::update_dto(&names),
        ),
        (entities_dir.join("mod.rs"), templates::entities_mod(&names)),
        (
            entities_dir.join(format!("{}.rs", names.snake)),
            templates::entity(&names),
        ),
    ];
    for (path, contents) in files {
        let state = write_generated(&path, &contents)?;
        record(&mut report, &path, state);
    }
    register_root_module(root, &names, &mut report)?;
    ensure_main_registration(root, &mut report);
    ensure_app_import(root, &names, &mut report);
    Ok(report)
}

/// Generates a custom parameter decorator.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_decorator(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_decorator.rs", names.snake),
        &templates::decorator(&names),
    )
}

/// Generates an exception filter.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_filter(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_filter.rs", names.snake),
        &templates::filter(&names),
    )
}

/// Generates a WebSocket gateway.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_gateway(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_gateway.rs", names.snake),
        &templates::gateway(&names),
    )
}

/// Generates a guard.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_guard(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_guard.rs", names.snake),
        &templates::guard(&names),
    )
}

/// Generates an interceptor.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_interceptor(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_interceptor.rs", names.snake),
        &templates::interceptor(&names),
    )
}

/// Generates middleware.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_middleware(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_middleware.rs", names.snake),
        &templates::middleware(&names),
    )
}

/// Generates a parameter pipe.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_pipe(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_pipe.rs", names.snake),
        &templates::pipe(&names),
    )
}

/// Generates an injectable provider.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names or conflicting files.
pub fn generate_provider(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    single_file(
        root,
        &format!("{}_provider.rs", names.snake),
        &templates::provider(&names),
    )
}

fn single_file(root: &Path, file_name: &str, contents: &str) -> Result<GenerationReport, CliError> {
    let mut report = GenerationReport::default();
    let path = root.join("src").join(file_name);
    let state = write_generated(&path, contents)?;
    record(&mut report, &path, state);
    Ok(report)
}

fn register_root_module(
    root: &Path,
    names: &Names,
    report: &mut GenerationReport,
) -> Result<(), CliError> {
    let registry = root.join("src/modules/mod.rs");
    let changed = ensure_items(&registry, &[&format!("pub mod {};", names.snake)])?;
    record(report, &registry, changed);
    Ok(())
}

fn ensure_main_registration(root: &Path, report: &mut GenerationReport) {
    let main = root.join("src/main.rs");
    if !main.is_file() {
        report
            .manual_instructions
            .push("add `mod modules;` to your crate root".to_owned());
        return;
    }
    if let Err(error) = ensure_items(&main, &["mod modules;"]) {
        report.manual_instructions.push(format!(
            "add `mod modules;` to `{}` ({error})",
            main.display()
        ));
    }
}

fn ensure_app_import(root: &Path, names: &Names, report: &mut GenerationReport) {
    let app = root.join("src/app.rs");
    let import = format!("crate::modules::{}::{}Module", names.snake, names.pascal);
    if !app.is_file() {
        report.manual_instructions.push(format!(
            "add `{import}` to your root module's `imports = [...]`"
        ));
        return;
    }
    match ensure_module_import(&app, &import) {
        Ok(changed) => record(report, &app, changed),
        Err(error) => report.manual_instructions.push(format!(
            "add `{import}` to `imports = [...]` in `{}` ({error})",
            app.display()
        )),
    }
}

/// Records a file operation outcome in a [`GenerationReport`].
pub(super) fn record(report: &mut GenerationReport, path: &Path, changed: bool) {
    if changed {
        report.created.push(path.to_owned());
    } else {
        report.unchanged.push(path.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::generators::GenerationReport;

    #[test]
    fn generation_report_default_is_empty() {
        let report = GenerationReport::default();
        assert!(report.created.is_empty());
        assert!(report.unchanged.is_empty());
        assert!(report.manual_instructions.is_empty());
    }

    #[test]
    fn record_created_files() {
        let mut report = GenerationReport::default();
        super::record(&mut report, &PathBuf::from("src/main.rs"), true);
        assert_eq!(report.created.len(), 1);
        assert!(report.unchanged.is_empty());
        assert_eq!(report.created[0].to_string_lossy(), "src/main.rs");
    }

    #[test]
    fn record_unchanged_files() {
        let mut report = GenerationReport::default();
        super::record(&mut report, &PathBuf::from("src/lib.rs"), false);
        assert_eq!(report.unchanged.len(), 1);
        assert!(report.created.is_empty());
    }

    #[test]
    fn single_file_generates_in_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        let report = super::single_file(root, "test_file.rs", "pub fn foo() {}").unwrap();
        assert_eq!(report.created.len(), 1);
        assert!(report.created[0].ends_with("src/test_file.rs"));
        assert!(root.join("src/test_file.rs").exists());
    }

    #[test]
    fn single_file_detects_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/conflict.rs"), "original content").unwrap();
        let result = super::single_file(root, "conflict.rs", "different content");
        assert!(result.is_err());
    }

    #[test]
    fn single_file_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        super::single_file(root, "idempotent.rs", "pub fn same() {}").unwrap();
        let report = super::single_file(root, "idempotent.rs", "pub fn same() {}").unwrap();
        assert_eq!(report.unchanged.len(), 1);
        assert!(report.created.is_empty());
    }

    #[test]
    fn register_root_module_adds_pub_mod() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/modules")).unwrap();
        let names = crate::generators::naming::Names::parse("my_module").unwrap();
        let mut report = GenerationReport::default();
        super::register_root_module(dir.path(), &names, &mut report).unwrap();
        let mod_rs = std::fs::read_to_string(dir.path().join("src/modules/mod.rs")).unwrap();
        assert!(mod_rs.contains("pub mod my_module;"));
        assert_eq!(report.created.len(), 1);
    }

    #[test]
    fn register_root_module_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/modules")).unwrap();
        let names = crate::generators::naming::Names::parse("my_module").unwrap();
        let mut report = GenerationReport::default();
        super::register_root_module(dir.path(), &names, &mut report).unwrap();
        let mut report2 = GenerationReport::default();
        super::register_root_module(dir.path(), &names, &mut report2).unwrap();
        assert!(report2.unchanged.len() == 1 || report2.created.is_empty());
    }

    #[test]
    fn generate_module_rejects_bad_names() {
        let dir = tempfile::tempdir().unwrap();
        assert!(super::generate_module(dir.path(), "123").is_err());
        assert!(super::generate_module(dir.path(), "mod").is_err());
        assert!(super::generate_module(dir.path(), "").is_err());
    }

    #[test]
    fn generate_module_creates_directory_structure() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let report = super::generate_module(dir.path(), "users").unwrap();
        assert!(!report.created.is_empty());
        assert!(dir.path().join("src/modules/users/mod.rs").exists());
    }

    #[test]
    fn generate_controller_adds_controller_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let report = super::generate_controller(dir.path(), "products").unwrap();
        assert!(!report.created.is_empty());
        assert!(dir.path().join("src/modules/products/controller").is_dir());
    }

    #[test]
    fn generate_service_adds_services_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let report = super::generate_service(dir.path(), "orders").unwrap();
        assert!(!report.created.is_empty());
        assert!(dir.path().join("src/modules/orders/services").is_dir());
    }

    #[test]
    fn generate_repository_adds_repositories_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let report = super::generate_repository(dir.path(), "inventory").unwrap();
        assert!(!report.created.is_empty());
        assert!(
            dir.path()
                .join("src/modules/inventory/repositories")
                .is_dir()
        );
    }

    #[test]
    fn generate_all_generator_artifacts_are_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let first = super::generate_resource(dir.path(), "articles").unwrap();
        let second = super::generate_resource(dir.path(), "articles").unwrap();

        // First run creates files
        assert!(!first.created.is_empty());
        // Second run should find all unchanged
        assert!(second.created.is_empty() || !second.unchanged.is_empty());
    }

    #[test]
    fn generate_resource_creates_full_structure() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let report = super::generate_resource(dir.path(), "articles").unwrap();

        let module_dir = dir.path().join("src/modules/articles");
        assert!(module_dir.join("mod.rs").exists());
        assert!(module_dir.join("controller").is_dir());
        assert!(module_dir.join("services").is_dir());
        assert!(module_dir.join("repositories").is_dir());
        assert!(module_dir.join("dto").is_dir());
        assert!(module_dir.join("entities").is_dir());
        assert!(module_dir.join("tests").is_dir());
        assert!(!report.manual_instructions.is_empty());
    }

    #[test]
    fn ensure_main_registration_adds_manual_instruction_when_no_main() {
        let dir = tempfile::tempdir().unwrap();
        let mut report = GenerationReport::default();
        super::ensure_main_registration(dir.path(), &mut report);
        assert!(!report.manual_instructions.is_empty());
        assert!(report.manual_instructions[0].contains("mod modules"));
    }

    #[test]
    fn ensure_app_import_adds_manual_instruction_when_no_app() {
        let dir = tempfile::tempdir().unwrap();
        let names = crate::generators::naming::Names::parse("test").unwrap();
        let mut report = GenerationReport::default();
        super::ensure_app_import(dir.path(), &names, &mut report);
        assert!(!report.manual_instructions.is_empty());
        assert!(report.manual_instructions[0].contains("crate::modules::test::TestModule"));
    }
}
