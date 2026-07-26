/// Microservice app and library crate generators.
pub mod app;
mod common;
mod graphql;
mod monorepo;
/// New-project scaffolding.
pub mod project;
/// Production-ready resource generators (authentication, authorization, etc.).
pub mod ready_resource;
/// Module, controller, service, resource, and single-file generators.
pub mod resource;

/// Generates a full authentication module.
pub use ready_resource::generate_ready_resource;
/// Generates a basic auth module (passwords + sessions).
pub use ready_resource::generate_ready_resource_basic;
/// Generates an email module with configurable delivery backends.
pub use ready_resource::generate_ready_resource_email;
/// Generates a file upload module with configurable storage backends.
pub use ready_resource::generate_ready_resource_file_upload;
/// Generates a JWT-only auth module.
pub use ready_resource::generate_ready_resource_jwt;
/// Generates an OAuth-only auth module.
pub use ready_resource::generate_ready_resource_oauth;

// Re-export all public generator functions
pub use app::{generate_app, generate_library};
pub use graphql::generate_graphql_resolver;
pub use resource::{
    generate_controller, generate_decorator, generate_filter, generate_gateway, generate_guard,
    generate_interceptor, generate_middleware, generate_module, generate_pipe, generate_provider,
    generate_repository, generate_resource, generate_service,
};

pub(crate) use common::naming::Names;

use std::path::{Path, PathBuf};

/// The kind of application to generate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppKind {
    /// Standard HTTP service with `AxumAdapter`.
    Http,
    /// gRPC service with `tonic`.
    Grpc,
    /// GraphQL service with `async-graphql`.
    Graphql,
}

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
        let report = super::resource::single_file(root, "test_file.rs", "pub fn foo() {}").unwrap();
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
        let result = super::resource::single_file(root, "conflict.rs", "different content");
        assert!(result.is_err());
    }

    #[test]
    fn single_file_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        super::resource::single_file(root, "idempotent.rs", "pub fn same() {}").unwrap();
        let report =
            super::resource::single_file(root, "idempotent.rs", "pub fn same() {}").unwrap();
        assert_eq!(report.unchanged.len(), 1);
        assert!(report.created.is_empty());
    }

    #[test]
    fn register_root_module_adds_pub_mod() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/modules")).unwrap();
        let names = crate::generators::common::naming::Names::parse("my_module").unwrap();
        let mut report = GenerationReport::default();
        super::resource::register_root_module(dir.path(), &names, &mut report).unwrap();
        let mod_rs = std::fs::read_to_string(dir.path().join("src/modules/mod.rs")).unwrap();
        assert!(mod_rs.contains("pub mod my_module;"));
        assert_eq!(report.created.len(), 1);
    }

    #[test]
    fn register_root_module_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/modules")).unwrap();
        let names = crate::generators::common::naming::Names::parse("my_module").unwrap();
        let mut report = GenerationReport::default();
        super::resource::register_root_module(dir.path(), &names, &mut report).unwrap();
        let mut report2 = GenerationReport::default();
        super::resource::register_root_module(dir.path(), &names, &mut report2).unwrap();
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

        assert!(!first.created.is_empty());
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
        super::resource::ensure_main_registration(dir.path(), &mut report);
        assert!(!report.manual_instructions.is_empty());
        assert!(report.manual_instructions[0].contains("mod modules"));
    }

    #[test]
    fn ensure_app_import_adds_manual_instruction_when_no_app() {
        let dir = tempfile::tempdir().unwrap();
        let names = crate::generators::common::naming::Names::parse("test").unwrap();
        let mut report = GenerationReport::default();
        super::resource::ensure_app_import(dir.path(), &names, &mut report);
        assert!(!report.manual_instructions.is_empty());
        assert!(report.manual_instructions[0].contains("crate::modules::test::TestModule"));
    }
}
