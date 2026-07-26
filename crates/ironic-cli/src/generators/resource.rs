use std::path::Path;

use crate::CliError;

use super::{
    GenerationReport, Names,
    common::{source, templates},
    record,
};

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
        source::write_module_shell(&module_dir.join("mod.rs"), &names.pascal)?,
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
        source::write_generated(&path, &templates::controller(&names))?,
    );
    source::write_generated(
        &controller_dir.join("mod.rs"),
        &templates::controller_mod(&names),
    )?;
    source::ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod controller;",
            &format!("pub use controller::{}Controller;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let controller_type = format!("{}Controller", names.pascal);
        if let Err(error) =
            source::ensure_module_array_item(&module_mod, &controller_type, "controllers")
        {
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
        source::write_generated(&path, &templates::repository(&names))?,
    );
    source::write_generated(
        &repos_dir.join("mod.rs"),
        &templates::repository_mod(&names),
    )?;
    source::ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod repositories;",
            &format!("pub use repositories::{}Repository;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let repo_type = format!("{}Repository", names.pascal);
        if let Err(error) = source::ensure_module_array_item(&module_mod, &repo_type, "providers") {
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
        source::write_generated(&path, &templates::service(&names))?,
    );
    source::write_generated(
        &services_dir.join("mod.rs"),
        &templates::services_mod(&names),
    )?;
    source::ensure_items(
        &module_dir.join("mod.rs"),
        &[
            "pub mod services;",
            &format!("pub use services::{}Service;", names.pascal),
        ],
    )?;
    let module_mod = module_dir.join("mod.rs");
    if module_mod.is_file() {
        let service_type = format!("{}Service", names.pascal);
        if let Err(error) =
            source::ensure_module_array_item(&module_mod, &service_type, "providers")
        {
            report.manual_instructions.push(format!(
                "add `{service_type}` to `providers = [...]` on `{}Module` ({error})",
                names.pascal
            ));
        }
    }
    Ok(report)
}

/// Generates a full CRUD resource vertical slice.
///
/// Creates the following structure inside `src/modules/{name}/`:
///
/// ```text
/// mod.rs              — root module with #[derive(Module)]
/// tests/
///   mod.rs            — test entry (declares unit + integration)
///   unit.rs           — business logic tests (no HTTP)
///   integration.rs    — full HTTP request/response tests
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
/// Also registers the module in `src/modules/mod.rs`, ensures `mod modules;` in
/// `main.rs`, and adds the import to the root `AppModule`.
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
        let state = source::write_generated(&path, &contents)?;
        record(&mut report, &path, state);
    }
    register_root_module(root, &names, &mut report)?;
    ensure_main_registration(root, &mut report);
    ensure_app_import(root, &names, &mut report);
    ensure_serde_dep(root, &mut report);
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

/// Writes a single generated Rust file under `src/<file_name>`.
///
/// Creates parent directories as needed. Returns [`GenerationReport`] with
/// the outcome. Errors on content conflict (file exists with different content).
pub(super) fn single_file(
    root: &Path,
    file_name: &str,
    contents: &str,
) -> Result<GenerationReport, CliError> {
    let mut report = GenerationReport::default();
    let path = root.join("src").join(file_name);
    let state = source::write_generated(&path, contents)?;
    record(&mut report, &path, state);
    Ok(report)
}

/// Adds `pub mod <name>;` to `src/modules/mod.rs`, creating the file if missing.
pub(super) fn register_root_module(
    root: &Path,
    names: &Names,
    report: &mut GenerationReport,
) -> Result<(), CliError> {
    let registry = root.join("src/modules/mod.rs");
    let changed = source::ensure_items(&registry, &[&format!("pub mod {};", names.snake)])?;
    record(report, &registry, changed);
    Ok(())
}

/// Ensures `mod modules;` exists in `src/main.rs`, adding a manual instruction if absent.
pub(super) fn ensure_main_registration(root: &Path, report: &mut GenerationReport) {
    let main = root.join("src/main.rs");
    if !main.is_file() {
        report
            .manual_instructions
            .push("add `mod modules;` to your crate root".to_owned());
        return;
    }
    if let Err(error) = source::ensure_items(&main, &["mod modules;"]) {
        report.manual_instructions.push(format!(
            "add `mod modules;` to `{}` ({error})",
            main.display()
        ));
    }
}

/// Ensures `serde` is in `Cargo.toml` (required for DTO/entity derives).
fn ensure_serde_dep(root: &Path, report: &mut GenerationReport) {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        return;
    }
    let content = std::fs::read_to_string(&manifest).unwrap_or_default();
    if content.contains("serde") {
        return;
    }
    let dep = "serde = { version = \"1\", features = [\"derive\"] }";
    let new_content = content.replace("[dependencies]\n", &format!("[dependencies]\n{dep}\n"));
    if new_content != content {
        std::fs::write(&manifest, new_content).ok();
        report
            .manual_instructions
            .push("`serde` auto-added to Cargo.toml".into());
    }
}

/// Ensures the generated module is imported into the root `AppModule` in `src/app.rs`.
pub(super) fn ensure_app_import(root: &Path, names: &Names, report: &mut GenerationReport) {
    let app = root.join("src/app.rs");
    let import = format!("crate::modules::{}::{}Module", names.snake, names.pascal);
    if !app.is_file() {
        report.manual_instructions.push(format!(
            "add `{import}` to your root module's `imports = [...]`"
        ));
        return;
    }
    match source::ensure_module_import(&app, &import) {
        Ok(changed) => record(report, &app, changed),
        Err(error) => report.manual_instructions.push(format!(
            "add `{import}` to `imports = [...]` in `{}` ({error})",
            app.display()
        )),
    }
}
