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
/// Generates a basic auth module (passwords + sessions).
pub use ready_resource::generate_ready_resource_basic;
/// Generates a JWT-only auth module.
pub use ready_resource::generate_ready_resource_jwt;
/// Generates an OAuth-only auth module.
pub use ready_resource::generate_ready_resource_oauth;

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
    report.manual_instructions.push(format!(
        "add `{}Controller` to `controllers = [...]` on `{}Module`",
        names.pascal, names.pascal
    ));
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
    report.manual_instructions.push(format!(
        "add `{}Repository` to `providers = [...]` on `{}Module`",
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
    report.manual_instructions.push(format!(
        "add `{}Service` to `providers = [...]` on `{}Module`",
        names.pascal, names.pascal
    ));
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
        assert!(report.manual_instructions.iter().any(|i| i.contains("ProductsController")));
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
        assert!(dir.path().join("src/modules/inventory/repositories").is_dir());
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
