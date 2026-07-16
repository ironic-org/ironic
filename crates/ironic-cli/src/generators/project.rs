use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{naming::Names, source::write_generated};
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
) -> Result<ProjectReport, CliError> {
    let names = Names::parse(name)?;
    let manifest = manifest(&names.kebab, framework_workspace);
    let files = [
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
            destination.join("docker-compose.yml"),
            docker_compose(&names.kebab),
        ),
        (destination.join("Makefile"), makefile().into()),
        (destination.join("justfile"), justfile().into()),
        (
            destination.join("rust-toolchain.toml"),
            rust_toolchain().into(),
        ),
        (destination.join("README.md"), readme(&names.kebab)),
        (destination.join("src/main.rs"), main_source(&names.kebab)),
        (destination.join("src/app.rs"), app_source().into()),
        (
            destination.join("src/welcome.rs"),
            welcome_source(&names.kebab),
        ),
        (
            destination.join("src/platform/mod.rs"),
            platform_mod().into(),
        ),
        (
            destination.join("src/platform/config.rs"),
            platform_config().into(),
        ),
        (
            destination.join("src/platform/database.rs"),
            platform_database(),
        ),
        (
            destination.join("src/platform/telemetry.rs"),
            platform_telemetry().into(),
        ),
        (destination.join("src/modules/mod.rs"), modules_mod().into()),
        (
            destination.join("src/modules/example/mod.rs"),
            example_module().into(),
        ),
        (
            destination.join("src/modules/example/controller/mod.rs"),
            example_controller_mod().into(),
        ),
        (
            destination.join("src/modules/example/controller/example_controller.rs"),
            example_controller().into(),
        ),
        (
            destination.join("src/modules/example/services/mod.rs"),
            example_service_mod().into(),
        ),
        (
            destination.join("src/modules/example/services/example_service.rs"),
            example_service().into(),
        ),
        (
            destination.join("src/modules/example/repositories/mod.rs"),
            example_repository_mod().into(),
        ),
        (
            destination.join("src/modules/example/repositories/example_repository.rs"),
            example_repository().into(),
        ),
        (
            destination.join("src/modules/example/dto/mod.rs"),
            example_dto_mod().into(),
        ),
        (
            destination.join("src/modules/example/dto/create_example_dto.rs"),
            example_create_dto().into(),
        ),
        (
            destination.join("src/modules/example/dto/update_example_dto.rs"),
            example_update_dto().into(),
        ),
        (
            destination.join("src/modules/example/entities/mod.rs"),
            example_entity_mod().into(),
        ),
        (
            destination.join("src/modules/example/entities/example.rs"),
            example_entity().into(),
        ),
        (
            destination.join("src/modules/example/tests/mod.rs"),
            example_test_mod().into(),
        ),
        (
            destination.join("src/modules/example/tests/unit.rs"),
            example_test_unit().into(),
        ),
        (
            destination.join("src/modules/example/tests/integration.rs"),
            example_test_integration().into(),
        ),
    ];
    let cidir = destination.join(".github/workflows");
    fs::create_dir_all(&cidir).map_err(|error| CliError::io("create directory", &cidir, error))?;
    fs::write(cidir.join("ci.yml"), ci_workflow())
        .map_err(|error| CliError::io("write", cidir.join("ci.yml"), error))?;

    // Validate all owned paths before writing. Allow pre-existing non-source files
    // (README.md, .gitignore, etc.) to be preserved; error on source file conflicts.
    let source_patterns = ["Cargo.toml", "ironic.toml", "src/", ".github/"];
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
        |workspace| {
            format!(
                "path = \"{}\", default-features = false",
                toml_path(workspace)
            )
        },
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
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
garde = "0.23"
sqlx = {{ version = "0.9", features = ["runtime-tokio", "postgres"] }}
tracing = {{ version = "0.1", features = ["attributes"] }}
dotenvy = "0.15"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

[dev-dependencies]
tokio = {{ version = "1", features = ["macros", "rt"] }}

# Available features (uncomment to enable):
# serialization   — role-based field exposure
# cache           — CacheInterceptor with InMemoryCache
# scheduling      — Fixed-interval and cron background tasks
# cron            — Cron expression scheduling
# realtime        — WebSocket gateways with rooms/broadcasting
# resilience      — Retry with backoff + circuit breaker
# telemetry       — Distributed tracing (OTLP)
# auth            — Password hashing, JWT, OAuth2, sessions
# distributed     — Queues, microservices, CQRS, sagas, gRPC, GraphQL
"#,
    )
}

// ── Project scaffolding ────────────────────────────────────────────────

fn main_source(name: &str) -> String {
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

#[ironic::main]
async fn main() {{
    dotenvy::dotenv().ok();
    platform::telemetry::init_tracing();

    let addr = format!(
        "{{}}:{{}}",
        platform::config::env("SERVER_HOST").unwrap_or_else(|| "0.0.0.0".into()),
        platform::config::env("SERVER_PORT").unwrap_or_else(|| "8080".into()),
    );
    let cors_origins = platform::config::env_json_array("CORS_ORIGINS");
    let rate_limit_max: u64 = platform::config::env_parsed("RATE_LIMIT_MAX", 100u64);

    let application = FrameworkApplication::builder()
        .module(AppModule::definition())
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
    )
}

fn app_source() -> &'static str {
    r"use ironic::prelude::*;
use crate::welcome::WelcomeModule;
use crate::modules::example::ExampleModule;
use ironic::metrics::MetricsModule;

#[derive(Module)]
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, ExampleModule])]
pub struct AppModule;
"
}

fn welcome_source(name: &str) -> String {
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
    )
}

fn modules_mod() -> &'static str {
    "pub mod example;\n"
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

// ── Example module (CRUD) ──────────────────────────────────────────────

fn example_module() -> &'static str {
    r"use ironic::prelude::*;

pub mod controller;
pub mod repositories;
pub mod services;
pub mod dto;
pub mod entities;

#[cfg(test)]
mod tests;

pub use controller::ExampleController;
pub use repositories::ExampleRepository;
pub use services::ExampleService;

#[derive(Module)]
#[module(providers = [ExampleRepository, ExampleService], controllers = [ExampleController])]
pub struct ExampleModule;
"
}

fn example_controller_mod() -> &'static str {
    "pub mod example_controller;\npub use example_controller::ExampleController;\n"
}

fn example_controller() -> &'static str {
    r#"use std::sync::Arc;
use ironic::prelude::*;
use super::super::services::ExampleService;
use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::entities::Example;

#[controller("/example")]
#[derive(Injectable)]
pub struct ExampleController { service: Arc<ExampleService> }

#[routes]
impl ExampleController {
    #[get]
    #[api(summary = "List all examples", tag = "Examples", security = "bearer")]
    #[resp(200, "A list of examples", json = Vec<Example>)]
    async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {
        Ok(Json(self.service.list()))
    }

    #[get("/:id")]
    #[api(summary = "Get an example by ID", tag = "Examples")]
    #[resp(200, "The requested example", json = Example)]
    #[resp(404, "Example not found")]
    async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {
        self.service.find(id).map(Json)
    }

    #[post]
    #[api(summary = "Create a new example", tag = "Examples")]
    #[req_body(json = CreateExampleDto)]
    #[resp(201, "Example created", json = Example)]
    #[resp(400, "Validation error")]
    async fn create(&self, #[body] dto: CreateExampleDto) -> Result<Json<Example>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }

    #[put("/:id")]
    #[api(summary = "Update an existing example", tag = "Examples")]
    #[req_body(json = UpdateExampleDto)]
    #[resp(200, "Example updated", json = Example)]
    #[resp(404, "Example not found")]
    async fn update(&self, #[param] id: u64, #[body] dto: UpdateExampleDto) -> Result<Json<Example>, HttpError> {
        self.service.update(id, dto).map(Json)
    }

    #[delete("/:id")]
    #[api(summary = "Delete an example", tag = "Examples")]
    #[resp(204, "Example deleted")]
    #[resp(404, "Example not found")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {
        self.service.delete(id)
    }
}
"#
}

fn example_service_mod() -> &'static str {
    "pub mod example_service;\npub use example_service::ExampleService;\n"
}

fn example_service() -> &'static str {
    r"use std::sync::Arc;
use ironic::prelude::*;
use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::entities::Example;
use crate::modules::example::repositories::ExampleRepository;

#[derive(Injectable)]
pub struct ExampleService {
    pub repository: Arc<ExampleRepository>,
}

impl ExampleService {
    pub fn list(&self) -> Vec<Example> {
        self.repository.list()
    }

    pub fn find(&self, id: u64) -> Result<Example, HttpError> {
        self.repository.find(id)
    }

    pub fn create(&self, dto: CreateExampleDto) -> Example {
        self.repository.create(dto.name, dto.description)
    }

    pub fn update(&self, id: u64, dto: UpdateExampleDto) -> Result<Example, HttpError> {
        self.repository.update(id, dto.name, dto.description)
    }

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {
        self.repository.delete(id)
    }
}
"
}

fn example_repository_mod() -> &'static str {
    "pub mod example_repository;\npub use example_repository::ExampleRepository;\n"
}

fn example_repository() -> &'static str {
    r#"use std::collections::HashMap;
use std::sync::Mutex;
use ironic::prelude::*;
use crate::modules::example::entities::Example;

static STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| Mutex::new(Store { items: HashMap::new(), next_id: 1 }));

struct Store { items: HashMap<u64, Example>, next_id: u64 }

#[derive(Injectable)]
pub struct ExampleRepository;

impl ExampleRepository {
    pub fn list(&self) -> Vec<Example> {
        STORE.lock().unwrap().items.values().cloned().collect()
    }

    pub fn find(&self, id: u64) -> Result<Example, HttpError> {
        STORE.lock().unwrap().items.get(&id).cloned()
            .ok_or_else(|| HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found")))
    }

    pub fn create(&self, name: String, description: Option<String>) -> Example {
        let mut store = STORE.lock().unwrap();
        let id = store.next_id;
        store.next_id += 1;
        let item = Example { id, name, description: description.unwrap_or_default() };
        store.items.insert(id, item.clone());
        item
    }

    pub fn update(&self, id: u64, name: Option<String>, description: Option<String>) -> Result<Example, HttpError> {
        let mut store = STORE.lock().unwrap();
        let item = store.items.get_mut(&id)
            .ok_or_else(|| HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found")))?;
        if let Some(name) = name { item.name = name; }
        if let Some(desc) = description { item.description = desc; }
        Ok(item.clone())
    }

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {
        STORE.lock().unwrap().items.remove(&id)
            .map(|_| ())
            .ok_or_else(|| HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found")))
    }
}
"#
}

fn example_dto_mod() -> &'static str {
    "pub mod create_example_dto;\npub mod update_example_dto;\npub use create_example_dto::CreateExampleDto;\npub use update_example_dto::UpdateExampleDto;\n"
}

fn example_create_dto() -> &'static str {
    r"use garde::Validate;
use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, OpenApiSchema)]
pub struct CreateExampleDto {
    #[garde(length(min = 1, max = 256))]
    /// Item name (1–256 characters).
    pub name: String,
    #[garde(skip)]
    /// Optional description.
    pub description: Option<String>,
}
"
}

fn example_update_dto() -> &'static str {
    r"use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]
pub struct UpdateExampleDto {
    /// New name (leave `null` to keep unchanged).
    pub name: Option<String>,
    /// New description (leave `null` to keep unchanged).
    pub description: Option<String>,
}
"
}

fn example_entity_mod() -> &'static str {
    "pub mod example;\npub use example::Example;\n"
}

fn example_entity() -> &'static str {
    r"use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]
pub struct Example {
    /// Unique identifier.
    pub id: u64,
    /// Item name.
    pub name: String,
    /// Item description.
    pub description: String,
}
"
}

// ── Example tests ──────────────────────────────────────────────────────

fn example_test_mod() -> &'static str {
    r"/// Unit tests — service and business logic in isolation (no HTTP).
#[cfg(test)]
mod unit;
/// Integration tests — full HTTP request/response through the framework.
#[cfg(test)]
mod integration;
"
}

fn example_test_unit() -> &'static str {
    r#"//! Unit tests for `ExampleService`.

use std::sync::Arc;
use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::repositories::ExampleRepository;
use crate::modules::example::services::ExampleService;

fn service() -> ExampleService {
    ExampleService { repository: Arc::new(ExampleRepository) }
}

#[test]
fn create_and_find() {
    let svc = service();
    let item = svc.create(CreateExampleDto { name: "Test".into(), description: None });
    assert_eq!(item.name, "Test");
    let found = svc.find(item.id).unwrap();
    assert_eq!(found.name, "Test");
}

#[test]
fn update_works() {
    let svc = service();
    let item = svc.create(CreateExampleDto { name: "Old".into(), description: None });
    let updated = svc.update(item.id, UpdateExampleDto { name: Some("New".into()), description: None }).unwrap();
    assert_eq!(updated.name, "New");
}

#[test]
fn delete_works() {
    let svc = service();
    let item = svc.create(CreateExampleDto { name: "Del".into(), description: None });
    assert!(svc.delete(item.id).is_ok());
    assert!(svc.find(item.id).is_err());
}

#[test]
fn not_found_error() {
    let svc = service();
    let err = svc.find(999).unwrap_err();
    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);
}

#[test]
fn list_works() {
    let svc = service();
    svc.create(CreateExampleDto { name: "A".into(), description: None });
    svc.create(CreateExampleDto { name: "B".into(), description: None });
    assert!(svc.list().len() >= 2);
}
"#
}

fn example_test_integration() -> &'static str {
    r#"//! Integration tests for Example — full HTTP request/response cycles.

use ironic::{HttpStatus, TestApplication};
use serde_json::json;

use super::super::*;

async fn app() -> TestApplication {
    TestApplication::new::<ExampleModule>().await.expect("test app must initialise")
}

#[tokio::test]
async fn list_returns_ok() {
    let a = app().await;
    assert_eq!(a.get("/example").send().await.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn create_and_get() {
    let a = app().await;
    let resp = a.post("/example").json(&json!({"name": "Test", "description": null})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let id = resp.json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    assert_eq!(a.get(&format!("/example/{id}")).send().await.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn update_works() {
    let a = app().await;
    let id = a.post("/example").json(&json!({"name": "Old"})).send().await
        .json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    let resp = a.put(&format!("/example/{id}")).json(&json!({"name": "New"})).send().await;
    assert_eq!(resp.json::<serde_json::Value>().unwrap()["name"], "New");
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn delete_works() {
    let a = app().await;
    let id = a.post("/example").json(&json!({"name": "Del"})).send().await
        .json::<serde_json::Value>().unwrap()["id"].as_u64().unwrap();
    a.delete(&format!("/example/{id}")).send().await;
    assert_eq!(a.get(&format!("/example/{id}")).send().await.status(), HttpStatus::NOT_FOUND);
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn not_found_returns_404() {
    let a = app().await;
    a.get("/example/999").send().await.assert_status(404);
    a.shutdown().await.unwrap();
}

// To enable request body validation error tests, uncomment the `garde` feature
// in Cargo.toml and add a test like:
// #[tokio::test]
// async fn create_rejects_empty_name() {
//     let a = app().await;
//     let resp = a.post("/example").json(&json!({"name": ""})).send().await;
//     assert_eq!(resp.status(), HttpStatus::BAD_REQUEST);
//     a.shutdown().await.unwrap();
// }
"#
}

// ── Platform ───────────────────────────────────────────────────────────

fn platform_mod() -> &'static str {
    "pub mod config;\npub mod telemetry;\n// pub mod database;\n"
}

fn platform_config() -> &'static str {
    r#"use std::env;

pub fn env(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn env_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key).ok()
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
}

fn platform_database() -> String {
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
//!    let row = sqlx::query("SELECT ...").fetch_one(db()).await?;
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
    .to_owned()
}

fn platform_telemetry() -> &'static str {
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
        r#"# Stage 1: Build
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Stage 2: Distroless runtime
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/{binary} /app/{binary}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
EXPOSE 8080
CMD ["./{binary}"]
"#,
    )
}

fn docker_compose(name: &str) -> String {
    format!(
        r#"services:
  app:
    build: .
    ports:
      - 8080:8080
    env_file: .env
    restart: unless-stopped
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: CHANGE_ME
      POSTGRES_DB: {name}
    ports:
      - 5432:5432
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d {name}"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - 6379:6379
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  pgdata:
"#,
    )
}

fn makefile() -> &'static str {
    r".PHONY: build test dev fmt clippy docker-build docker-up docker-down clean

build:
	cargo build

test:
	cargo test -- --test-threads=1

dev:
	cargo run -- dev

fmt:
	cargo fmt --all

clippy:
	cargo clippy -- -D warnings

docker-build:
	docker build -t app .

docker-up:
	docker compose up -d

docker-down:
	docker compose down

clean:
	cargo clean
"
}

fn justfile() -> &'static str {
    r"build:
    cargo build

test:
    cargo test -- --test-threads=1

dev:
    cargo run -- dev

fmt:
    cargo fmt --all

clippy:
    cargo clippy -- -D warnings

docker-build:
    docker build -t app .

docker-up:
    docker compose up -d

docker-down:
    docker compose down

clean:
    cargo clean
"
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
# Install Ironic CLI
cargo install ironic

# Run with hot reload
ironic dev

# Or run directly
cargo run
```

Open http://localhost:8080 in your browser.

## Commands

| Task | Command |
|------|--------|
| Start dev server | `make dev` |
| Run tests | `make test` |
| Build | `make build` |
| Format | `make fmt` |
| Lint | `make clippy` |

## Docker

```bash
make docker-up    # Start app + postgres + redis
make docker-down  # Stop everything
make docker-build # Build image only
```

## Endpoints

| Path | Description |
|------|-------------|
| `GET /` | Welcome JSON |
| `GET /health` | Health check |
| `GET /docs` | Swagger UI |
| `GET /example` | Example CRUD |

## Environment

Copy `.env.example` to `.env` and adjust values.
",
    )
}

fn ci_workflow() -> &'static str {
    r"name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test -- --test-threads=1
"
}

// ── Helpers ────────────────────────────────────────────────────────────

fn toml_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}
