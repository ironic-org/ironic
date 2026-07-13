mod naming;
/// New-project scaffolding.
pub mod project;
mod source;
mod templates;

use std::path::{Path, PathBuf};

use naming::Names;
use source::{ensure_items, ensure_module_import, write_generated, write_module_shell};

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
    let file_name = format!("{}_controller.rs", names.snake);
    let path = module_dir.join(&file_name);
    record(
        &mut report,
        &path,
        write_generated(&path, &templates::controller(&names))?,
    );
    ensure_items(
        &module_dir.join("mod.rs"),
        &[
            &format!("pub mod {}_controller;", names.snake),
            &format!(
                "pub use {}_controller::{}Controller;",
                names.snake, names.pascal
            ),
        ],
    )?;
    report.manual_instructions.push(format!(
        "add `{}Controller` to `controllers = [...]` on `{}Module`",
        names.pascal, names.pascal
    ));
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
    let path = module_dir.join(format!("{}_service.rs", names.snake));
    record(
        &mut report,
        &path,
        write_generated(&path, &templates::service(&names))?,
    );
    ensure_items(
        &module_dir.join("mod.rs"),
        &[
            &format!("pub mod {}_service;", names.snake),
            &format!("pub use {}_service::{}Service;", names.snake, names.pascal),
        ],
    )?;
    report.manual_instructions.push(format!(
        "add `{}Service` to `providers = [...]` on `{}Module`",
        names.pascal, names.pascal
    ));
    Ok(report)
}

/// Generates a complete module, service, and controller vertical slice.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, conflicting files, or unsafe source edits.
pub fn generate_resource(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let names = Names::parse(name)?;
    let module_dir = root.join("src/modules").join(&names.snake);
    let mut report = GenerationReport::default();
    let files = [
        (
            module_dir.join("mod.rs"),
            templates::resource_module(&names),
        ),
        (
            module_dir.join(format!("{}_service.rs", names.snake)),
            templates::service(&names),
        ),
        (
            module_dir.join(format!("{}_controller.rs", names.snake)),
            templates::resource_controller(&names),
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

fn record(report: &mut GenerationReport, path: &Path, changed: bool) {
    if changed {
        report.created.push(path.to_owned());
    } else {
        report.unchanged.push(path.to_owned());
    }
}
