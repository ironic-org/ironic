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

    let files: Vec<(std::path::PathBuf, String)> = vec![
        (dest.join("Cargo.toml"), app_manifest(&names)),
        (dest.join("src/main.rs"), app_main(&names)),
        (dest.join("src/app.rs"), app_module(&names)),
        (
            dest.join("src/lib.rs"),
            "pub mod app;\npub mod modules;\npub use app::AppModule;\n".to_string(),
        ),
        (dest.join("src/modules/mod.rs"), "pub mod health;\n".into()),
        (
            dest.join("src/modules/health/mod.rs"),
            r"use ironic::prelude::*;

#[derive(Module)]
#[module()]
pub struct HealthModule;
"
            .into(),
        ),
        (
            dest.join("src/modules/health/controller/mod.rs"),
            "pub mod health_controller;\n".into(),
        ),
        (
            dest.join("src/modules/health/controller/health_controller.rs"),
            r#"use ironic::prelude::*;

#[controller("/health")]
pub struct HealthController;

#[routes]
impl HealthController {
    #[get("/")]
    async fn check(&self) -> Json<serde_json::Value> {
        Json(serde_json::json!({"status": "ok"}))
    }
}
"#
            .into(),
        ),
    ];

    for (path, contents) in &files {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CliError::Io {
                action: "create directory",
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let changed = source::write_generated(path, contents)?;
        record(&mut report, path, changed);
    }

    // Update workspace Cargo.toml to include the new app
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

    report.manual_instructions.push(format!(
        "run `cd apps/{} && cargo run` to start the service",
        names.kebab
    ));

    Ok(report)
}

fn app_manifest(names: &naming::Names) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
description = "Microservice generated by Ironic"

[dependencies]
ironic = {{ workspace = true }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[lib]
name = "{name}"
"#,
        name = names.kebab
    )
}

fn app_main(names: &naming::Names) -> String {
    format!(
        r#"use {name}::app::AppModule;
use ironic::prelude::*;

#[ironic::main]
async fn main() -> Result<(), anyhow::Error> {{
    tracing_subscriber::fmt::init();

    Application::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await?
        .listen("0.0.0.0:3000")
        .await?;

    Ok(())
}}
"#,
        name = names.kebab
    )
}

fn app_module(_names: &naming::Names) -> String {
    r"use ironic::prelude::*;

pub struct AppModule;

impl Module for AppModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .build()
    }
}
"
    .to_string()
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

/// Creates shared library directories in a monorepo workspace.
fn create_monorepo_libs(root: &Path) -> Result<(), CliError> {
    for lib in &["shared-config", "proto", "observability"] {
        let lib_dir = root.join("libs").join(lib);
        let lib_dir_src = lib_dir.join("src");
        fs::create_dir_all(&lib_dir_src).map_err(|e| CliError::Io {
            action: "create lib directory",
            path: lib_dir.clone(),
            source: e,
        })?;
        let lib_cargo = format!(
            r#"[package]
name = "{lib}"
version = "0.1.0"
edition = "2024"
"#
        );
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
    }
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
        r#"use ::ironic::prelude::*;

pub struct {name}Module;

impl Module for {name}Module {{
    fn definition() -> ModuleDefinition {{
        ModuleDefinition::builder("{name}")
            .build()
    }}
}}
"#,
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
