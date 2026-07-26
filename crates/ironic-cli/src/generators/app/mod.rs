mod graphql;
mod grpc;
pub(crate) mod http;
mod platform;
pub(crate) mod production;

pub(crate) use http::{app_controller, app_service};
pub(crate) use production::app_production_guide;

use std::path::Path;

use crate::CliError;

use super::{
    AppKind, GenerationReport,
    common::{naming, source},
    monorepo,
};

/// Generates a new microservice app inside a monorepo workspace.
///
/// When `grpc` is true, generates a gRPC service with `tonic` + `prost`
/// instead of an HTTP service with `AxumAdapter`.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, existing destinations, or filesystem errors.
pub fn generate_app(root: &Path, name: &str, kind: AppKind) -> Result<GenerationReport, CliError> {
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
    let files = match kind {
        AppKind::Grpc => generate_grpc_files(&dest, &names, port),
        AppKind::Graphql => generate_graphql_files(&dest, &names, port),
        AppKind::Http => generate_app_files(&dest, &names, port),
    };

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
    let version = env!("CARGO_PKG_VERSION");
    vec![
        (dest.join("Cargo.toml"), http::app_manifest(names)),
        (dest.join("Dockerfile"), http::app_dockerfile(names, port)),
        (dest.join(".env"), http::app_env(names, port)),
        (dest.join("src/main.rs"), http::app_main(names, port)),
        (dest.join("src/app.rs"), http::app_module().to_string()),
        (
            dest.join("src/app_controller.rs"),
            http::app_controller().to_string(),
        ),
        (
            dest.join("src/app_service.rs"),
            http::app_service(&names.raw, version),
        ),
        (
            dest.join("src/platform/mod.rs"),
            platform::app_platform_mod(),
        ),
        (
            dest.join("src/platform/logging.rs"),
            platform::app_platform_logging(),
        ),
        (
            dest.join("src/platform/config.rs"),
            platform::app_platform_config(),
        ),
        (
            dest.join("PRODUCTION.md"),
            production::app_production_guide(&names.raw, port),
        ),
    ]
}

/// Returns the list of files for a gRPC app.
fn generate_grpc_files(
    dest: &Path,
    names: &naming::Names,
    port: u16,
) -> Vec<(std::path::PathBuf, String)> {
    let version = env!("CARGO_PKG_VERSION");
    vec![
        (dest.join("Cargo.toml"), grpc::app_manifest_grpc(names)),
        (dest.join("Dockerfile"), http::app_dockerfile(names, port)),
        (dest.join(".env"), http::app_env(names, port)),
        (dest.join("src/main.rs"), grpc::app_main_grpc(names, port)),
        (dest.join("src/app.rs"), grpc::app_module_grpc()),
        (
            dest.join("src/app_service.rs"),
            http::app_service(&names.raw, version),
        ),
        (
            dest.join("src/platform/mod.rs"),
            platform::app_platform_mod(),
        ),
        (
            dest.join("src/platform/logging.rs"),
            platform::app_platform_logging(),
        ),
        (
            dest.join("src/platform/config.rs"),
            platform::app_platform_config(),
        ),
        (dest.join("build.rs"), grpc::app_build()),
        (dest.join("proto/hello.proto"), grpc::app_proto(names)),
        (dest.join("src/modules/mod.rs"), grpc::app_modules_mod()),
        (
            dest.join("src/modules/greet/mod.rs"),
            grpc::app_greet_mod().to_string(),
        ),
        (
            dest.join("src/modules/greet/greeter_service.rs"),
            grpc::app_greeter_service(names),
        ),
        (
            dest.join("src/modules/greet/greet_repository.rs"),
            grpc::app_greet_repository(),
        ),
        (
            dest.join("PRODUCTION.md"),
            production::app_production_guide(&names.raw, port),
        ),
    ]
}

/// Returns the list of files for a GraphQL app.
fn generate_graphql_files(
    dest: &Path,
    names: &naming::Names,
    port: u16,
) -> Vec<(std::path::PathBuf, String)> {
    vec![
        (
            dest.join("Cargo.toml"),
            graphql::app_manifest_graphql(names),
        ),
        (dest.join("Dockerfile"), http::app_dockerfile(names, port)),
        (dest.join(".env"), http::app_env(names, port)),
        (
            dest.join("src/main.rs"),
            graphql::app_main_graphql(names, port),
        ),
        (dest.join("src/app.rs"), graphql::app_module_graphql()),
        (
            dest.join("src/app_service.rs"),
            graphql::app_service_graphql(),
        ),
        (
            dest.join("src/platform/mod.rs"),
            platform::app_platform_mod(),
        ),
        (
            dest.join("src/platform/logging.rs"),
            platform::app_platform_logging(),
        ),
        (
            dest.join("src/platform/config.rs"),
            platform::app_platform_config(),
        ),
        (
            dest.join("PRODUCTION.md"),
            production::app_production_guide(&names.raw, port),
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
    super::record(report, path, changed);
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

/// Generates a reusable library crate with an Ironic module scaffold.
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
        super::record(&mut report, path, changed);
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
