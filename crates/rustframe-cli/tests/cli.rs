//! End-to-end contracts for command parsing and deterministic generation.

use std::{fs, process::Command};

use clap::Parser;
use rustframe_cli::{
    cli::{Cli, Command as CliCommand, Generator},
    generators::{
        generate_controller, generate_module, generate_resource, generate_service, project,
    },
};

#[test]
fn parses_primary_commands_and_generator_aliases() {
    assert!(matches!(
        Cli::try_parse_from(["rustframe", "new", "demo"])
            .unwrap()
            .command,
        CliCommand::New(_)
    ));
    assert!(matches!(
        Cli::try_parse_from(["rustframe", "g", "co", "users"])
            .unwrap()
            .command,
        CliCommand::Generate(arguments)
            if matches!(arguments.generator, Generator::Controller(_))
    ));
    assert!(matches!(
        Cli::try_parse_from(["rustframe", "build", "--", "--release"])
            .unwrap()
            .command,
        CliCommand::Build(arguments) if arguments.cargo_args == ["--release"]
    ));
}

#[test]
fn rejects_names_that_cannot_form_rust_identifiers() {
    let temporary = tempfile::tempdir().unwrap();
    assert!(generate_module(temporary.path(), "123").is_err());
    assert!(generate_module(temporary.path(), "mod").is_err());
}

#[test]
fn generators_are_idempotent_and_register_rust_modules() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        root.join("src/app.rs"),
        "use rustframe::prelude::*;\n#[derive(Module)]\n#[module()]\nstruct AppModule;\n",
    )
    .unwrap();

    generate_module(root, "users").unwrap();
    generate_controller(root, "users").unwrap();
    generate_service(root, "users").unwrap();
    let second = generate_service(root, "users").unwrap();

    assert!(!second.unchanged.is_empty());
    let registry = fs::read_to_string(root.join("src/modules/mod.rs")).unwrap();
    assert_eq!(registry.matches("pub mod users;").count(), 1);
    let module = fs::read_to_string(root.join("src/modules/users/mod.rs")).unwrap();
    assert_eq!(module.matches("pub mod users_service;").count(), 1);
    assert_eq!(module.matches("pub mod users_controller;").count(), 1);
    let main = fs::read_to_string(root.join("src/main.rs")).unwrap();
    assert_eq!(main.matches("mod modules;").count(), 1);
    let app = fs::read_to_string(root.join("src/app.rs")).unwrap();
    assert_eq!(app.matches("crate::modules::users::UsersModule").count(), 1);
}

#[test]
fn unsafe_source_edits_produce_manual_instructions() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "this is not rust").unwrap();

    let report = generate_module(root, "users").unwrap();

    assert!(
        report
            .manual_instructions
            .iter()
            .any(|instruction| instruction.contains("mod modules"))
    );
    assert_eq!(
        fs::read_to_string(root.join("src/main.rs")).unwrap(),
        "this is not rust"
    );
}

#[test]
fn generated_project_builds_and_tests_offline() {
    let temporary = tempfile::tempdir().unwrap();
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap();
    let destination = temporary.path().join("sample-api");
    project::create(&destination, "sample-api", Some(workspace)).unwrap();

    let first = generate_resource(&destination, "products").unwrap();
    let second = generate_resource(&destination, "products").unwrap();
    assert!(!first.created.is_empty());
    assert!(second.created.is_empty());
    let app = fs::read_to_string(destination.join("src/app.rs")).unwrap();
    assert_eq!(
        app.matches("crate::modules::products::ProductsModule")
            .count(),
        1
    );

    let status = Command::new("cargo")
        .args(["test", "--offline", "--manifest-path"])
        .arg(destination.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", temporary.path().join("target"))
        .status()
        .unwrap();
    assert!(
        status.success(),
        "generated project did not pass cargo test"
    );
}
