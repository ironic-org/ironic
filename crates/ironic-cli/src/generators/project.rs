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
        (destination.join(".env.example"), dotenv_example()),
        (destination.join(".gitignore"), gitignore()),
        (destination.join("Dockerfile"), dockerfile()),
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
    let dep_spec = workspace.map_or_else(
        || format!("version = \"{}\"", env!("CARGO_PKG_VERSION")),
        |workspace| {
            format!(
                "path = \"{}\", default-features = false",
                toml_path(workspace)
            )
        },
    );
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.97\"\npublish = false\n\n[dependencies]\nironic = {{ features = [\"validation\"], {dep_spec} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\ngarde = \"0.23\"\ndotenvy = \"0.15\"\n\n[dev-dependencies]\ntokio = {{ version = \"1\", features = [\"macros\", \"rt\"] }}\n\n# Available features (uncomment to enable):\n# security        — CORS, rate limiting, security headers, CSRF\n# versioning      — URI, header, and media-type API versioning\n# serialization   — role-based field exposure\n# cache           — CacheInterceptor with InMemoryCache\n# scheduling      — Fixed-interval and cron background tasks\n# cron            — Cron expression scheduling\n# realtime        — WebSocket gateways with rooms/broadcasting\n# compression     — gzip, brotli, zstd response compression\n# metrics         — Prometheus endpoint + request counters\n# resilience      — Retry with backoff + circuit breaker\n# telemetry       — Distributed tracing (OTLP)\n# database        — SQLx, SeaORM, Diesel (postgres/mysql/sqlite)\n# auth            — Password hashing, JWT, OAuth2, sessions\n# distributed     — Queues, microservices, CQRS, sagas, gRPC, GraphQL\n# openapi         — Auto-generate OpenAPI schemas + Swagger UI\n",
    )
}

fn main_source(name: &str) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "mod app;\nmod modules;\nmod welcome;\n\nuse std::env;\n\nuse ironic::{{AxumAdapter, prelude::*}};\n\nuse app::AppModule;\n\n#[ironic::main]\nasync fn main() {{\n    dotenvy::dotenv().ok();\n    let host = env::var(\"SERVER_HOST\").unwrap_or_else(|_| \"127.0.0.1\".into());\n    let port = env::var(\"SERVER_PORT\").unwrap_or_else(|_| \"3000\".into());\n    let addr = format!(\"{{}}:{{}}\", host, port);\n\n    let application = FrameworkApplication::builder()\n        .module(AppModule::definition())\n        .platform(AxumAdapter::new())\n        .build()\n        .await\n        .expect(\"application must initialise\");\n\n    println!(\"🚀 {name} → http://{{}} (ironic v{version})\", addr);\n\n    application\n        .listen(&addr)\n        .await\n        .expect(\"application server failed\");\n}}\n"
    )
}

fn app_source() -> String {
    "use ironic::prelude::*;\nuse crate::welcome::WelcomeModule;\nuse crate::modules::example::ExampleModule;\n\n#[derive(Module)]\n#[module(imports = [HealthModule, WelcomeModule, ExampleModule])]\npub struct AppModule;\n".to_owned()
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
    "use ironic::prelude::*;\n\npub mod controller;\npub mod services;\npub mod dto;\npub mod entities;\n\n#[cfg(test)]\nmod tests;\n\npub use controller::ExampleController;\npub use services::ExampleService;\n\n#[derive(Module)]\n#[module(providers = [ExampleService], controllers = [ExampleController])]\npub struct ExampleModule;\n".to_owned()
}

fn example_controller_mod() -> String {
    "pub mod example_controller;\npub use example_controller::ExampleController;\n".to_owned()
}

fn example_controller() -> String {
    "use std::sync::Arc;\nuse ironic::prelude::*;\nuse super::super::services::ExampleService;\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::entities::Example;\n\n#[controller(\"/example\")]\n#[derive(Injectable)]\npub struct ExampleController { service: Arc<ExampleService> }\n\n#[routes]\nimpl ExampleController {\n    #[get]\n    async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> { Ok(Json(self.service.list())) }\n\n    #[get(\"/:id\")]\n    async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> { self.service.find(id).map(Json) }\n\n    #[post]\n    async fn create(&self, #[body] dto: CreateExampleDto) -> Result<Json<Example>, HttpError> { Ok(Json(self.service.create(dto))) }\n\n    #[put(\"/:id\")]\n    async fn update(&self, #[param] id: u64, #[body] dto: UpdateExampleDto) -> Result<Json<Example>, HttpError> { self.service.update(id, dto).map(Json) }\n\n    #[delete(\"/:id\")]\n    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> { self.service.delete(id) }\n}\n".to_owned()
}

fn example_service_mod() -> String {
    "pub mod example_service;\npub use example_service::ExampleService;\n".to_owned()
}

fn example_service() -> String {
    "use std::collections::HashMap;\nuse std::sync::Mutex;\nuse ironic::prelude::*;\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::entities::Example;\n\n#[derive(Injectable)]\npub struct ExampleService;\n\nstatic STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| Mutex::new(Store { items: HashMap::new(), next_id: 1 }));\n\nstruct Store { items: HashMap<u64, Example>, next_id: u64 }\n\nimpl ExampleService {\n    pub fn list(&self) -> Vec<Example> { STORE.lock().unwrap().items.values().cloned().collect() }\n\n    pub fn find(&self, id: u64) -> Result<Example, HttpError> {\n        STORE.lock().unwrap().items.get(&id).cloned()\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))\n    }\n\n    pub fn create(&self, dto: CreateExampleDto) -> Example {\n        let mut store = STORE.lock().unwrap();\n        let id = store.next_id;\n        store.next_id += 1;\n        let item = Example { id, name: dto.name, description: dto.description.unwrap_or_default() };\n        store.items.insert(id, item.clone());\n        item\n    }\n\n    pub fn update(&self, id: u64, dto: UpdateExampleDto) -> Result<Example, HttpError> {\n        let mut store = STORE.lock().unwrap();\n        let item = store.items.get_mut(&id)\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))?;\n        if let Some(name) = dto.name { item.name = name; }\n        if let Some(desc) = dto.description { item.description = desc; }\n        Ok(item.clone())\n    }\n\n    pub fn delete(&self, id: u64) -> Result<(), HttpError> {\n        STORE.lock().unwrap().items.remove(&id)\n            .map(|_| ())\n            .ok_or_else(|| HttpError::not_found(\"EXAMPLE_NOT_FOUND\", format!(\"Item {id} not found\")))\n    }\n}\n".to_owned()
}

fn example_dto_mod() -> String {
    "pub mod create_example_dto;\npub mod update_example_dto;\npub use create_example_dto::CreateExampleDto;\npub use update_example_dto::UpdateExampleDto;\n".to_owned()
}

fn example_create_dto() -> String {
    "use garde::Validate;\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize, Validate)]\npub struct CreateExampleDto {\n    #[garde(length(min = 1, max = 256))]\n    pub name: String,\n    #[garde(skip)]\n    pub description: Option<String>,\n}\n".to_owned()
}

fn example_update_dto() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct UpdateExampleDto {\n    pub name: Option<String>,\n    pub description: Option<String>,\n}\n".to_owned()
}

fn example_entity_mod() -> String {
    "pub mod example;\npub use example::Example;\n".to_owned()
}

fn example_entity() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Example {\n    pub id: u64,\n    pub name: String,\n    pub description: String,\n}\n".to_owned()
}

fn example_test_mod() -> String {
    "/// Unit tests — service and business logic in isolation (no HTTP).\n#[cfg(test)]\nmod unit;\n/// Integration tests — full HTTP request/response through the framework.\n#[cfg(test)]\nmod integration;\n".to_owned()
}

fn example_test_unit() -> String {
    "//! Unit tests for `ExampleService`.\n\nuse crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};\nuse crate::modules::example::services::ExampleService;\n\n#[test]\nfn create_and_find() {\n    let svc = ExampleService;\n    let item = svc.create(CreateExampleDto { name: \"Test\".into(), description: None });\n    assert_eq!(item.name, \"Test\");\n    let found = svc.find(item.id).unwrap();\n    assert_eq!(found.name, \"Test\");\n}\n\n#[test]\nfn update_works() {\n    let svc = ExampleService;\n    let item = svc.create(CreateExampleDto { name: \"Old\".into(), description: None });\n    let updated = svc.update(item.id, UpdateExampleDto { name: Some(\"New\".into()), description: None }).unwrap();\n    assert_eq!(updated.name, \"New\");\n}\n\n#[test]\nfn delete_works() {\n    let svc = ExampleService;\n    let item = svc.create(CreateExampleDto { name: \"Del\".into(), description: None });\n    assert!(svc.delete(item.id).is_ok());\n    assert!(svc.find(item.id).is_err());\n}\n\n#[test]\nfn not_found_error() {\n    let svc = ExampleService;\n    let err = svc.find(999).unwrap_err();\n    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);\n}\n\n#[test]\nfn list_works() {\n    let svc = ExampleService;\n    svc.create(CreateExampleDto { name: \"A\".into(), description: None });\n    svc.create(CreateExampleDto { name: \"B\".into(), description: None });\n    assert!(svc.list().len() >= 2);\n}\n".to_owned()
}

fn example_test_integration() -> String {
    "//! Integration tests for Example — full HTTP request/response cycles.\n\nuse ironic::{HttpStatus, TestApplication};\nuse serde_json::json;\n\nuse super::super::*;\n\nasync fn app() -> TestApplication {\n    TestApplication::new::<ExampleModule>().await.expect(\"test app must initialise\")\n}\n\n#[tokio::test]\nasync fn list_returns_ok() {\n    let a = app().await;\n    assert_eq!(a.get(\"/example\").send().await.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn create_and_get() {\n    let a = app().await;\n    let resp = a.post(\"/example\").json(&json!({\"name\": \"Test\", \"description\": null})).send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    let id = resp.json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    assert_eq!(a.get(&format!(\"/example/{id}\")).send().await.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn update_works() {\n    let a = app().await;\n    let id = a.post(\"/example\").json(&json!({\"name\": \"Old\"})).send().await\n        .json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    let resp = a.put(&format!(\"/example/{id}\")).json(&json!({\"name\": \"New\"})).send().await;\n    assert_eq!(resp.json::<serde_json::Value>().unwrap()[\"name\"], \"New\");\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn delete_works() {\n    let a = app().await;\n    let id = a.post(\"/example\").json(&json!({\"name\": \"Del\"})).send().await\n        .json::<serde_json::Value>().unwrap()[\"id\"].as_u64().unwrap();\n    a.delete(&format!(\"/example/{id}\")).send().await;\n    assert_eq!(a.get(&format!(\"/example/{id}\")).send().await.status(), HttpStatus::NOT_FOUND);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn not_found_returns_404() {\n    let a = app().await;\n    a.get(\"/example/999\").send().await.assert_status(404);\n    a.shutdown().await.unwrap();\n}\n".to_owned()
}

// ── Infrastructure templates ──────────────────────────────────────────

fn dotenv_example() -> String {
    "# Server\nSERVER_HOST=0.0.0.0\nSERVER_PORT=3000\n\n# Logging\nRUST_LOG=info\n\n# Database (uncomment to use)\n# DATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/mydb\n\n# Redis (uncomment to use)\n# REDIS_URL=redis://localhost:6379\n".to_owned()
}

fn gitignore() -> String {
    "/target\n**/*.rs.bk\n.env\n*.log\n.DS_Store\n*.pdb\n".to_owned()
}

fn dockerfile() -> String {
    "# Stage 1: Build\nFROM rust:1.97-slim-bookworm AS builder\nWORKDIR /app\nCOPY Cargo.toml Cargo.lock* ./\nCOPY src ./src\nRUN cargo build --release --locked\n\n# Stage 2: Distroless runtime\nFROM gcr.io/distroless/cc-debian12\nCOPY --from=builder /app/target/release/* /app/\nEXPOSE 3000\nCMD [\"./app\"]\n".to_owned()
}

fn docker_compose(name: &str) -> String {
    format!(
        "services:\n  app:\n    build: .\n    ports:\n      - 3000:3000\n    env_file: .env\n    restart: unless-stopped\n    depends_on:\n      postgres:\n        condition: service_healthy\n      redis:\n        condition: service_healthy\n\n  postgres:\n    image: postgres:16-alpine\n    environment:\n      POSTGRES_USER: user\n      POSTGRES_PASSWORD: CHANGE_ME\n      POSTGRES_DB: {name}\n    ports:\n      - 5432:5432\n    volumes:\n      - pgdata:/var/lib/postgresql/data\n    healthcheck:\n      test: [\"CMD-SHELL\", \"pg_isready -U user -d {name}\"]\n      interval: 5s\n      timeout: 5s\n      retries: 5\n\n  redis:\n    image: redis:7-alpine\n    ports:\n      - 6379:6379\n    healthcheck:\n      test: [\"CMD\", \"redis-cli\", \"ping\"]\n      interval: 5s\n      timeout: 3s\n      retries: 5\n\nvolumes:\n  pgdata:\n"
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
        "# {name}\n\nBuilt with [Ironic](https://github.com/ironic-org/ironic) v{version}.\n\n## Quick start\n\n```bash\n# Install Ironic CLI\ncargo install ironic\n\n# Run with hot reload\nironic dev\n\n# Or run directly\ncargo run\n```\n\nOpen http://localhost:3000 in your browser.\n\n## Commands\n\n| Task | Command |\n|------|--------|\n| Start dev server | `make dev` |\n| Run tests | `make test` |\n| Build | `make build` |\n| Format | `make fmt` |\n| Lint | `make clippy` |\n\n## Docker\n\n```bash\nmake docker-up    # Start app + postgres + redis\nmake docker-down  # Stop everything\nmake docker-build # Build image only\n```\n\n## Endpoints\n\n| Path | Description |\n|------|-------------|\n| `GET /` | Welcome JSON |\n| `GET /health` | Health check |\n| `GET /example` | Example CRUD |\n\n## Environment\n\nCopy `.env.example` to `.env` and adjust values.\n"
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
