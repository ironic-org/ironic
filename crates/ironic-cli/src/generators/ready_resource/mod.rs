#![allow(clippy::needless_raw_string_hashes)]

use std::path::Path;

use crate::CliError;

use super::{
    GenerationReport,
    common::source::{ensure_items, ensure_module_import, write_generated},
};

/// Generates a full authentication module with passwords, JWT, OAuth, sessions, and RBAC.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules").join(name);
    let mut report = GenerationReport::default();

    let files = auth::auth_full_files(&module_dir, name);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, name, &mut report);
    Ok(report)
}

/// Generates a basic auth module (passwords + sessions only).
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_basic(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth::auth_basic_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

/// Generates a JWT-only auth module.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_jwt(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth::auth_jwt_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

/// Generates an OAuth-only auth module.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_oauth(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth::auth_oauth_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

fn register_module(root: &Path, name: &str, report: &mut GenerationReport) {
    let registry = root.join("src/modules/mod.rs");
    if let Err(e) = ensure_items(&registry, &[&format!("pub mod {name};")]) {
        report.manual_instructions.push(format!(
            "add `pub mod {name};` to {}: {e}",
            registry.display()
        ));
    } else {
        super::record(report, &registry, true);
    }

    let app = root.join("src/app.rs");
    let pascal = "Auth";
    let import = format!("crate::modules::{name}::{pascal}Module");
    if app.is_file() {
        if let Err(e) = ensure_module_import(&app, &import) {
            report.manual_instructions.push(format!(
                "add `{import}` to `imports = [...]` in {}: {e}",
                app.display()
            ));
        } else {
            super::record(report, &app, true);
        }
    } else {
        report.manual_instructions.push(format!(
            "add `{import}` to your root module's `imports = [...]`"
        ));
    }

    // Auto-add required dependencies to Cargo.toml
    let manifest = root.join("Cargo.toml");
    if manifest.is_file() {
        let mut content = std::fs::read_to_string(&manifest).unwrap_or_default();
        let deps = [
            ("jsonwebtoken", "jsonwebtoken = \"9\""),
            ("argon2", "argon2 = \"0.5\""),
            ("oauth2", "oauth2 = \"5.0\""),
            ("getrandom", "getrandom = \"0.4\""),
        ];
        let mut added = false;
        for (name, dep) in &deps {
            if !content.contains(name) {
                content = content.replace("[dependencies]\n", &format!("[dependencies]\n{dep}\n"));
                added = true;
            }
        }
        if added {
            std::fs::write(&manifest, content).ok();
            report.manual_instructions.push(
                "Dependencies auto-added to Cargo.toml. Run `cargo build` to fetch them.".into(),
            );
        }
    }
}

mod auth;
mod email;
mod file_upload;

pub use email::generate_ready_resource_email;
pub use file_upload::generate_ready_resource_file_upload;
