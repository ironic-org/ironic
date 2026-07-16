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
        (destination.join(".gitignore"), gitignore()),
        (destination.join("Dockerfile"), dockerfile(&names.kebab)),
        (
            destination.join("docker-compose.yml"),
            docker_compose(&names.kebab),
        ),
        (destination.join("Makefile"), makefile()),
        (destination.join("justfile"), justfile()),
        (destination.join("rust-toolchain.toml"), rust_toolchain()),
        (destination.join("README.md"), readme(&names.kebab)),
        (destination.join("src/main.rs"), main_source(&names.kebab)),
        (destination.join("src/app.rs"), app_source()),
        (
            destination.join("src/welcome.rs"),
            welcome_source(&names.kebab),
        ),
        (destination.join("src/platform/mod.rs"), platform_mod()),
        (
            destination.join("src/platform/config.rs"),
            platform_config(),
        ),
        (
            destination.join("src/platform/database.rs"),
            platform_database(),
        ),
        (
            destination.join("src/platform/telemetry.rs"),
            platform_telemetry(),
        ),
        (destination.join("src/modules/mod.rs"), modules_mod()),
        (
            destination.join("src/modules/example/mod.rs"),
            example_module(),
        ),
        (
            destination.join("src/modules/example/controller/mod.rs"),
            example_controller_mod(),
        ),
        (
            destination.join("src/modules/example/controller/example_controller.rs"),
            example_controller(),
        ),
        (
            destination.join("src/modules/example/services/mod.rs"),
            example_service_mod(),
        ),
        (
            destination.join("src/modules/example/services/example_service.rs"),
            example_service(),
        ),
        (
            destination.join("src/modules/example/repositories/mod.rs"),
            example_repository_mod(),
        ),
        (
            destination.join("src/modules/example/repositories/example_repository.rs"),
            example_repository(),
        ),
        (
            destination.join("src/modules/example/dto/mod.rs"),
            example_dto_mod(),
        ),
        (
            destination.join("src/modules/example/dto/create_example_dto.rs"),
            example_create_dto(),
        ),
        (
            destination.join("src/modules/example/dto/update_example_dto.rs"),
            example_update_dto(),
        ),
        (
            destination.join("src/modules/example/entities/mod.rs"),
            example_entity_mod(),
        ),
        (
            destination.join("src/modules/example/entities/example.rs"),
            example_entity(),
        ),
        (
            destination.join("src/modules/example/tests/mod.rs"),
            example_test_mod(),
        ),
        (
            destination.join("src/modules/example/tests/unit.rs"),
            example_test_unit(),
        ),
        (
            destination.join("src/modules/example/tests/integration.rs"),
            example_test_integration(),
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

fn manifest(name: &str, workspace: Option<&Path>) -> String {
    // Use a MAJOR.MINOR semver range (e.g. "0.4") so generated projects resolve
    // to the latest published 0.4.x version on crates.io, regardless of the CLI
    // tool's own version (which may be a locally-bumped unpublished version).
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
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.97\"\npublish = false\n\n[dependencies]\nironic = {{ features = [\"security\", \"compression\", \"metrics\", \"validation\", \"versioning\", \"openapi\", \"logging\", \"sqlx-postgres\"], {dep_spec} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\ngarde = \"0.23\"\nsqlx = {{ version = \"0.9\", features = [\"runtime-tokio\", \"postgres\"] }}\ntracing = {{ version = \"0.1\", features = [\"attributes\"] }}\ndotenvy = \"0.15\"\ntracing-subscriber = {{ version = \"0.3\", features = [\"env-filter\"] }}\n\n[dev-dependencies]\ntokio = {{ version = \"1\", features = [\"macros\", \"rt\"] }}\n\n# Available features (uncomment to enable):\n# serialization   — role-based field exposure\n# cache           — CacheInterceptor with InMemoryCache\n# scheduling      — Fixed-interval and cron background tasks\n# cron            — Cron expression scheduling\n# realtime        — WebSocket gateways with rooms/broadcasting\n# resilience      — Retry with backoff + circuit breaker\n# telemetry       — Distributed tracing (OTLP)\n# auth            — Password hashing, JWT, OAuth2, sessions\n# distributed     — Queues, microservices, CQRS, sagas, gRPC, GraphQL\n",
    )
}

fn main_source(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "mod app;\nmod modules;\nmod platform;\nmod welcome;\n\nuse std::time::Duration;\n\nuse ironic::{{AxumAdapter, OpenApiConfig, OpenApiAxumExt}};\nuse ironic::metrics::{{MetricsLayer, MetricsConfig}};\nuse ironic::prelude::*;\nuse ironic::security::{{\n    CorsConfig, CorsMiddleware,\n    RateLimitMiddleware,\n    SecurityHeadersConfig, SecurityHeadersMiddleware,\n}};\n\nuse app::AppModule;\n\n#[ironic::main]\nasync fn main() {{\n    dotenvy::dotenv().ok();\n    platform::telemetry::init_tracing();\n\n    let addr = format!(\n        \"{{}}:{{}}\",\n        platform::config::env(\"SERVER_HOST\").unwrap_or_else(|| \"0.0.0.0\".into()),\n        platform::config::env(\"SERVER_PORT\").unwrap_or_else(|| \"8080\".into()),\n    );\n    let cors_origins = platform::config::env_json_array(\"CORS_ORIGINS\");\n    let rate_limit_max: u64 = platform::config::env_parsed(\"RATE_LIMIT_MAX\", 100u64);\n\n    let application = FrameworkApplication::builder()\n        .module(AppModule::definition())\n        .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))\n        .middleware(RateLimitMiddleware::new(rate_limit_max, 60))\n        .middleware(CorsMiddleware::new(CorsConfig::new().allowed_origins(cors_origins)))\n        .platform(\n            AxumAdapter::new()\n                .compression()\n                .request_body_limit(5 * 1024 * 1024)\n                .request_timeout(Duration::from_secs(30))\n                .configure_router(|r| {{\n                    r.layer(MetricsLayer::new(MetricsConfig::default()))\n                }})\n                .with_openapi(OpenApiConfig::new(\"{name}\", \"0.1.0\"))\n                .swagger_ui(\"/docs\"),\n        )\n        .build()\n        .await\n        .expect(\"application must initialise\");\n\n    println!(\"🚀 {name} → http://{{}} (ironic v{version})\", addr);\n\n    application\n        .listen(&addr)\n        .await\n        .expect(\"application server failed\");\n}}\n",
    )
}

fn app_source() -> String {
    "use ironic::prelude::*;\nuse crate::welcome::WelcomeModule;\nuse crate::modules::example::ExampleModule;\nuse ironic::metrics::MetricsModule;\n\n#[derive(Module)]\n#[module(imports = [HealthModule, MetricsModule, WelcomeModule, ExampleModule])]\npub struct AppModule;\n".to_owned()
}

fn welcome_source(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "use ironic::prelude::*;\n\n#[controller(\"/\")]\n#[derive(Injectable)]\nstruct WelcomeController;\n\n#[routes]\nimpl WelcomeController {{\n    #[get]\n    async fn index(&self) -> Result<Json<serde_json::Value>, HttpError> {{\n        Ok(Json(serde_json::json!({{\n            \"name\": \"{name}\",\n            \"framework\": \"Ironic\",\n            \"version\": \"{version}\",\n            \"status\": \"running\",\n            \"health\": \"/health\",\n            \"docs\": \"/docs\"\n        }})))\n    }}\n}}\n\n#[derive(Module)]\n#[module(controllers = [WelcomeController])]\npub struct WelcomeModule;\n"
    )
}

fn modules_mod() -> String {
    "pub mod example;\n".to_owned()
}

fn project_config(name: &str) -> String {
    format!(
        "[project]\nname = \"{name}\"\nsource_root = \"src\"\ndefault_module = \"src/app.rs\"\n\n[generate]\nmodule_path = \"src/modules\"\n"
    )
}

// ── Example CRUD module ───────────────────────────────────────────────

fn example_module() -> String {
    "use ironic::prelude::*;\n\npub mod controller;\npub mod repositories;\npub mod services;\npub mod dto;\npub mod entities;\n\n#[cfg(test)]\nmod tests;\n\npub use controller::ExampleController;\npub use repositories::ExampleRepository;\npub use services::ExampleService;\n\n#[derive(Module)]\n#[module(providers = [ExampleRepository, ExampleService], controllers = [ExampleController])]\npub struct ExampleModule;\n".to_owned()
}

fn example_controller_mod() -> String {
    "pub mod example_controller;\npub use example_controller::ExampleController;\n".to_owned()
}

fn example_controller() -> String {
    "use std::sync::Arc;\nuse ironic::prelude::*;\nuse super::super::services::ExampleService;\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::entities::Example;\n\n#[controller(\"/example\")]\n#[derive(Injectable)]\npub struct ExampleController { service: Arc<ExampleService> }\n\n#[routes]\nimpl ExampleController {\n    #[get]\n    async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {\n        Ok(Json(self.service.list()))\n    }\n\n    #[get(\"/:id\")]\n    async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {\n        self.service.find(id).map(Json)\n    }\n\n    #[post]\n    async fn create(&self, #[body] dto: CreateExampleDto) -> Result<Json<Example>, HttpError> {\n        Ok(Json(self.service.create(dto)))\n    }\n\n    #[put(\"/:id\")]\n    async fn update(&self, #[param] id: u64, #[body] dto: UpdateExampleDto) -> Result<Json<Example>, HttpError> {\n        self.service.update(id, dto).map(Json)\n    }\n\n    #[delete(\"/:id\")]\n    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {\n        self.service.delete(id)\n    }\n}\n".to_owned()
}

fn example_service_mod() -> String {
    "pub mod example_service;\npub use example_service::ExampleService;\n".to_owned()
}

fn example_service() -> String {
    "use std::sync::Arc;\nuse ironic::prelude::*;\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::entities::Example;\nuse crate::modules::example::repositories::ExampleRepository;\n\n#[derive(Injectable)]\npub struct ExampleService {\n    pub repository: Arc<ExampleRepository>,\n}\n\nimpl ExampleService {\n    pub fn list(&self) -> Vec<Example> {\n        self.repository.list()\n    }\n\n    pub fn find(&self, id: u64) -> Result<Example, HttpError> {\n        self.repository.find(id)\n    }\n\n    pub fn create(&self, dto: CreateExampleDto) -> Example {\n        self.repository.create(dto.name, dto.description)\n    }\n\n    pub fn update(&self, id: u64, dto: UpdateExampleDto) -> Result<Example, HttpError> {\n        self.repository.update(id, dto.name, dto.description)\n    }\n\n    pub fn delete(&self, id: u64) -> Result<(), HttpError> {\n        self.repository.delete(id)\n    }\n}\n".to_owned()
}

fn example_dto_mod() -> String {
    "pub mod create_example_dto;\npub mod update_example_dto;\npub use create_example_dto::CreateExampleDto;\npub use update_example_dto::UpdateExampleDto;\n".to_owned()
}

fn example_create_dto() -> String {
    "use garde::Validate;\nuse ironic::OpenApiSchema;\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize, Validate, OpenApiSchema)]\npub struct CreateExampleDto {\n    #[garde(length(min = 1, max = 256))]\n    /// Item name (1–256 characters).\n    pub name: String,\n    #[garde(skip)]\n    /// Optional description.\n    pub description: Option<String>,\n}\n".to_owned()
}

fn example_update_dto() -> String {
    "use ironic::OpenApiSchema;\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]\npub struct UpdateExampleDto {\n    /// New name (leave `null` to keep unchanged).\n    pub name: Option<String>,\n    /// New description (leave `null` to keep unchanged).\n    pub description: Option<String>,\n}\n".to_owned()
}

fn example_entity_mod() -> String {
    "pub mod example;\npub use example::Example;\n".to_owned()
}

fn example_entity() -> String {
    "use ironic::OpenApiSchema;\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize, OpenApiSchema)]\npub struct Example {\n    /// Unique identifier.\n    pub id: u64,\n    /// Item name.\n    pub name: String,\n    /// Item description.\n    pub description: String,\n}\n".to_owned()
}

fn example_test_mod() -> String {
    "/// Unit tests — service and business logic in isolation (no HTTP).\n#[cfg(test)]\nmod unit;\n/// Integration tests — full HTTP request/response through the framework.\n#[cfg(test)]\nmod integration;\n".to_owned()
}

fn example_test_unit() -> String {
    "//! Unit tests for `ExampleService`.\n\nuse std::sync::Arc;\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::repositories::ExampleRepository;\nuse crate::modules::example::services::ExampleService;\n\nfn service() -> ExampleService {\n    ExampleService { repository: Arc::new(ExampleRepository) }\n}\n\n#[test]\nfn create_and_find() {\n    let svc = service();\n    let item = svc.create(CreateExampleDto { name: \"Test\".into(), description: None });\n    assert_eq!(item.name, \"Test\");\n    let found = svc.find(item.id).unwrap();\n    assert_eq!(found.name, \"Test\");\n}\n\n#[test]\nfn update_works() {\n    let svc = service();\n    let item = svc.create(CreateExampleDto { name: \"Old\".into(), description: None });\n    let updated = svc.update(item.id, UpdateExampleDto { name: Some(\"New\".into()), description: None }).unwrap();\n    assert_eq!(updated.name, \"New\");\n}\n\n#[test]\nfn delete_works() {\n    let svc = service();\n    let item = svc.create(CreateExampleDto { name: \"Del\".into(), description: None });\n    assert!(svc.delete(item.id).is_ok());\n    assert!(svc.find(item.id).is_err());\n}\n\n#[test]\nfn not_found_error() {\n    let svc = service();\n    let err = svc.find(999).unwrap_err();\n    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);\n}\n\n#[test]\nfn list_works() {\n    let svc = service();\n    svc.create(CreateExampleDto { name: \"A\".into(), description: None });\n    svc.create(CreateExampleDto { name: \"B\".into(), description: None });\n    assert!(svc.list().len() >= 2);\n}\n".to_owned()
}

fn example_test_integration() -> String {
    "//! Integration tests for Example — full HTTP request/response cycles.\n\nuse ironic::{HttpStatus, TestApplication};\nuse serde_json::json;\n\nuse super::super::*;\n\nasync fn app() -> TestApplication {\n    TestApplication::new::<ExampleModule>().await.expect(\"test app must initialise\")\n}\n\n#[tokio::test]\nasync fn list_returns_ok() {\n    let a = app().await;\n    assert_eq!(a.get(\"/example\").send().await.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn create_and_get() {\n    let a = app().await;\n    let resp = a.post(\"/example\").json(&json!({\"name\": \"Test\", \"description\": null})).send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    let id = resp.json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    assert_eq!(a.get(&format!(\"/example/{id}\")).send().await.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn update_works() {\n    let a = app().await;\n    let id = a.post(\"/example\").json(&json!({\"name\": \"Old\"})).send().await\n        .json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    let resp = a.put(&format!(\"/example/{id}\")).json(&json!({\"name\": \"New\"})).send().await;\n    assert_eq!(resp.json::<serde_json::Value>().unwrap()[\"name\"], \"New\");\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn delete_works() {\n    let a = app().await;\n    let id = a.post(\"/example\").json(&json!({\"name\": \"Del\"})).send().await\n        .json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    a.delete(&format!(\"/example/{id}\")).send().await;\n    assert_eq!(a.get(&format!(\"/example/{id}\")).send().await.status(), HttpStatus::NOT_FOUND);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn not_found_returns_404() {\n    let a = app().await;\n    a.get(\"/example/999\").send().await.assert_status(404);\n    a.shutdown().await.unwrap();\n}\n\n// To enable request body validation, wire ValidationPipe in your controller:\n//   #[controller(\"/example\")]\n//   #[pipe(ValidationPipe)]\n// The CreateExampleDto already has garde validation rules defined.\n".to_owned()
}

// ── Platform templates ────────────────────────────────────────────────

fn platform_mod() -> String {
    "pub mod config;\npub mod telemetry;\n// pub mod database;\n".to_owned()
}

fn platform_config() -> String {
    "use std::env;\n\npub fn env(key: &str) -> Option<String> {\n    env::var(key).ok()\n}\n\npub fn env_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {\n    env::var(key).ok()\n        .and_then(|v| v.parse().ok())\n        .unwrap_or(default)\n}\n\npub fn env_json_array(key: &str) -> Vec<String> {\n    env::var(key)\n        .ok()\n        .and_then(|v| serde_json::from_str(&v).ok())\n        .unwrap_or_default()\n}\n\n#[allow(dead_code)]
pub fn server_address() -> String {\n    let host = env(\"SERVER_HOST\").unwrap_or_else(|| \"0.0.0.0\".into());\n    let port = env(\"SERVER_PORT\").unwrap_or_else(|| \"8080\".into());\n    format!(\"{host}:{port}\")\n}\n".to_owned()
}

fn platform_database() -> String {
    "//! Database connection pool (PostgreSQL via SQLx).
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
//!    let row = sqlx::query(\"SELECT ...\").fetch_one(db()).await?;
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
        .expect(\"DATABASE_URL must be set and pool initialized\")
}

#[allow(dead_code)]
pub async fn build_pool() -> sqlx::PgPool {
    let url = dotenvy::var(\"DATABASE_URL\").expect(\"DATABASE_URL must be set\");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(super::config::env(\"DB_POOL_SIZE\")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10))
        .connect(&url)
        .await
        .expect(\"failed to connect to database\");

    sqlx::migrate::Migrator::new(std::path::Path::new(\"./migrations\"))
        .await
        .expect(\"invalid migrations directory\")
        .run(&pool)
        .await
        .expect(\"failed to run migrations\");

    tracing::info!(\"database pool ready (max: {})\", pool.size());
    pool
}\n"
    .to_owned()
}

fn platform_telemetry() -> String {
    "use tracing_subscriber::EnvFilter;\n\npub fn init_tracing() {\n    tracing_subscriber::fmt()\n        .with_env_filter(\n            EnvFilter::try_from_default_env()\n                .unwrap_or_else(|_| EnvFilter::new(\"info\")),\n        )\n        .with_target(true)\n        .with_thread_ids(true)\n        .with_file(true)\n        .with_line_number(true)\n        .compact()\n        .init();\n}\n".to_owned()
}

// ── Repository templates ──────────────────────────────────────────────

fn example_repository_mod() -> String {
    "pub mod example_repository;\npub use example_repository::ExampleRepository;\n".to_owned()
}

fn example_repository() -> String {
    "use std::collections::HashMap;\nuse std::sync::Mutex;\nuse ironic::prelude::*;\nuse crate::modules::example::entities::Example;\n\nstatic STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| Mutex::new(Store { items: HashMap::new(), next_id: 1 }));\n\nstruct Store { items: HashMap<u64, Example>, next_id: u64 }\n\n#[derive(Injectable)]\npub struct ExampleRepository;\n\nimpl ExampleRepository {\n    pub fn list(&self) -> Vec<Example> {\n        STORE.lock().unwrap().items.values().cloned().collect()\n    }\n\n    pub fn find(&self, id: u64) -> Result<Example, HttpError> {\n        STORE.lock().unwrap().items.get(&id).cloned()\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))\n    }\n\n    pub fn create(&self, name: String, description: Option<String>) -> Example {\n        let mut store = STORE.lock().unwrap();\n        let id = store.next_id;\n        store.next_id += 1;\n        let item = Example { id, name, description: description.unwrap_or_default() };\n        store.items.insert(id, item.clone());\n        item\n    }\n\n    pub fn update(&self, id: u64, name: Option<String>, description: Option<String>) -> Result<Example, HttpError> {\n        let mut store = STORE.lock().unwrap();\n        let item = store.items.get_mut(&id)\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))?;\n        if let Some(name) = name { item.name = name; }\n        if let Some(desc) = description { item.description = desc; }\n        Ok(item.clone())\n    }\n\n    pub fn delete(&self, id: u64) -> Result<(), HttpError> {\n        STORE.lock().unwrap().items.remove(&id)\n            .map(|_| ())\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))\n    }\n}\n".to_owned()
}

// ── Infrastructure templates ──────────────────────────────────────────

fn dotenv_example(name: &str) -> String {
    format!(
        "# Server\nSERVER_HOST=0.0.0.0\nSERVER_PORT=8080\n\n# Logging\nRUST_LOG=info\n\n# Security ──────────────────────────────────────────────\n# Security headers (HSTS, CSP, X-Frame-Options, etc.) are always on with\n# secure defaults. You can customise them in src/main.rs.\n#\n# JSON array of allowed origins; leave empty to deny all cross-origin requests\n# Example: CORS_ORIGINS=[\"https://app.com\",\"https://admin.com\"]\nCORS_ORIGINS=[]\n# Maximum requests per IP per 60-second window\nRATE_LIMIT_MAX=100\n\n# Database\nDATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/{name}\n\n# Redis (uncomment to use)\n# REDIS_URL=redis://localhost:6379\n"
    )
}

fn gitignore() -> String {
    "/target\n**/*.rs.bk\n.env\n*.log\n.DS_Store\n*.pdb\n".to_owned()
}

fn dockerfile(name: &str) -> String {
    let binary = name.replace('-', "_");
    format!(
        "# Stage 1: Build\nFROM rust:1.97-slim-bookworm AS builder\nWORKDIR /app\nCOPY Cargo.toml Cargo.lock* ./\nCOPY src ./src\nRUN cargo build --release\n\n# Stage 2: Distroless runtime\nFROM gcr.io/distroless/cc-debian12\nWORKDIR /app\nCOPY --from=builder /app/target/release/{binary} /app/{binary}\nENV SERVER_HOST=0.0.0.0\nENV SERVER_PORT=8080\nEXPOSE 8080\nCMD [\"./{binary}\"]\n"
    )
}

fn docker_compose(name: &str) -> String {
    format!(
        "services:\n  app:\n    build: .\n    ports:\n      - 8080:8080\n    env_file: .env\n    restart: unless-stopped\n    depends_on:\n      postgres:\n        condition: service_healthy\n      redis:\n        condition: service_healthy\n\n  postgres:\n    image: postgres:16-alpine\n    environment:\n      POSTGRES_USER: user\n      POSTGRES_PASSWORD: CHANGE_ME\n      POSTGRES_DB: {name}\n    ports:\n      - 5432:5432\n    volumes:\n      - pgdata:/var/lib/postgresql/data\n    healthcheck:\n      test: [\"CMD-SHELL\", \"pg_isready -U user -d {name}\"]\n      interval: 5s\n      timeout: 5s\n      retries: 5\n\n  redis:\n    image: redis:7-alpine\n    ports:\n      - 6379:6379\n    healthcheck:\n      test: [\"CMD\", \"redis-cli\", \"ping\"]\n      interval: 5s\n      timeout: 3s\n      retries: 5\n\nvolumes:\n  pgdata:\n"
    )
}

fn makefile() -> String {
    ".PHONY: build test dev fmt clippy docker-build docker-up docker-down clean\n\nbuild:\n\tcargo build\n\ntest:\n\tcargo test -- --test-threads=1\n\ndev:\n\tcargo run -- dev\n\nfmt:\n\tcargo fmt --all\n\nclippy:\n\tcargo clippy -- -D warnings\n\ndocker-build:\n\tdocker build -t app .\n\ndocker-up:\n\tdocker compose up -d\n\ndocker-down:\n\tdocker compose down\n\nclean:\n\tcargo clean\n".to_owned()
}

fn justfile() -> String {
    "build:\n    cargo build\n\ntest:\n    cargo test -- --test-threads=1\n\ndev:\n    cargo run -- dev\n\nfmt:\n    cargo fmt --all\n\nclippy:\n    cargo clippy -- -D warnings\n\ndocker-build:\n    docker build -t app .\n\ndocker-up:\n    docker compose up -d\n\ndocker-down:\n    docker compose down\n\nclean:\n    cargo clean\n".to_owned()
}

fn rust_toolchain() -> String {
    "[toolchain]\nchannel = \"1.97\"\ncomponents = [\"rustfmt\", \"clippy\"]\n".to_owned()
}

fn readme(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "# {name}\n\nBuilt with [Ironic](https://github.com/ironic-org/ironic) v{version}.\n\n## Quick start\n\n```bash\n# Install Ironic CLI\ncargo install ironic\n\n# Run with hot reload\nironic dev\n\n# Or run directly\ncargo run\n```\n\nOpen http://localhost:8080 in your browser.\n\n## Commands\n\n| Task | Command |\n|------|--------|\n| Start dev server | `make dev` |\n| Run tests | `make test` |\n| Build | `make build` |\n| Format | `make fmt` |\n| Lint | `make clippy` |\n\n## Docker\n\n```bash\nmake docker-up    # Start app + postgres + redis\nmake docker-down  # Stop everything\nmake docker-build # Build image only\n```\n\n## Endpoints\n\n| Path | Description |\n|------|-------------|\n| `GET /` | Welcome JSON |\n| `GET /health` | Health check |\n| `GET /docs` | Swagger UI |\n| `GET /example` | Example CRUD |\n\n## Environment\n\nCopy `.env.example` to `.env` and adjust values.\n"
    )
}

fn ci_workflow() -> String {
    "name: CI\n\non:\n  push:\n    branches: [main]\n  pull_request:\n    branches: [main]\n\nenv:\n  CARGO_TERM_COLOR: always\n\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: dtolnay/rust-toolchain@stable\n        with:\n          components: rustfmt, clippy\n      - uses: Swatinem/rust-cache@v2\n      - run: cargo fmt --all -- --check\n      - run: cargo clippy -- -D warnings\n      - run: cargo test -- --test-threads=1\n".to_owned()
}

fn toml_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}
