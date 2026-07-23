#![doc = include_str!("../README.md")]

/// Command-line argument definitions.
pub mod cli;
mod commands;
mod error;
/// Deterministic project and source generators.
pub mod generators;

use std::io::{self, Write};

pub use error::CliError;

/// Parses process arguments and executes the selected command.
///
/// # Errors
///
/// Returns [`CliError`] when project generation, environment inspection, or Cargo execution fails.
///
/// # Example
///
/// ```no_run
/// ironic::run().ok();
/// ```
pub fn run() -> Result<(), CliError> {
    use clap::Parser;

    run_with(cli::Cli::parse(), &mut io::stdout())
}

/// Executes an already-parsed command and writes user-facing output to `output`.
///
/// # Errors
///
/// Returns [`CliError`] when the selected command fails.
///
/// # Example
///
/// ```
/// use clap::Parser;
/// use ironic::cli::{Cli, Command};
///
/// let cli = Cli::try_parse_from(["ironic", "doctor"]).unwrap();
/// let mut buf = Vec::new();
/// ironic::run_with(cli, &mut buf).ok(); // may fail if rustc is not installed
/// assert!(!buf.is_empty());
/// ```
pub fn run_with(command: cli::Cli, output: &mut impl Write) -> Result<(), CliError> {
    commands::execute(command, output)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Command, MigrateAction};

    #[test]
    fn run_with_dispatches_new_command() {
        let cli = Cli::try_parse_from(["ironic", "new", "test-proj"]).unwrap();
        assert!(matches!(cli.command, Command::New(_)));
    }

    #[test]
    fn run_with_dispatches_doctor_command() {
        let cli = Cli::try_parse_from(["ironic", "doctor"]).unwrap();
        assert!(matches!(cli.command, Command::Doctor));
    }

    #[test]
    fn run_with_dispatches_update_command() {
        let cli = Cli::try_parse_from(["ironic", "update"]).unwrap();
        assert!(matches!(cli.command, Command::Update));
    }

    #[test]
    fn run_with_dispatches_uninstall_command() {
        let cli = Cli::try_parse_from(["ironic", "uninstall"]).unwrap();
        assert!(matches!(cli.command, Command::Uninstall));
    }

    #[test]
    fn run_with_dispatches_start_command() {
        let cli = Cli::try_parse_from(["ironic", "start"]).unwrap();
        assert!(matches!(cli.command, Command::Start(_)));
    }

    #[test]
    fn run_with_dispatches_build_command() {
        let cli = Cli::try_parse_from(["ironic", "build", "--", "--release"]).unwrap();
        assert!(matches!(cli.command, Command::Build(a) if a.cargo_args == ["--release"]));
    }

    #[test]
    fn run_with_dispatches_test_command() {
        let cli = Cli::try_parse_from(["ironic", "test"]).unwrap();
        assert!(matches!(cli.command, Command::Test(_)));
    }

    #[test]
    fn run_with_dispatches_workspace_command() {
        let cli = Cli::try_parse_from(["ironic", "workspace", "."]).unwrap();
        assert!(matches!(cli.command, Command::Workspace(_)));
    }

    #[test]
    fn run_with_dispatches_routes_command() {
        let cli = Cli::try_parse_from(["ironic", "routes", "."]).unwrap();
        assert!(matches!(cli.command, Command::Routes(_)));
    }

    #[test]
    fn run_with_dispatches_graph_command() {
        let cli = Cli::try_parse_from(["ironic", "graph", "."]).unwrap();
        assert!(matches!(cli.command, Command::Graph(_)));
    }

    #[test]
    fn run_with_dispatches_generate_command() {
        let cli = Cli::try_parse_from(["ironic", "generate", "co", "test"]).unwrap();
        assert!(matches!(cli.command, Command::Generate(_)));
    }

    #[test]
    fn run_with_dispatches_generator_alias() {
        let cli = Cli::try_parse_from(["ironic", "g", "mo", "test"]).unwrap();
        assert!(matches!(cli.command, Command::Generate(_)));
    }

    #[test]
    fn run_with_dispatches_update_alias() {
        let cli = Cli::try_parse_from(["ironic", "upgrade"]).unwrap();
        assert!(matches!(cli.command, Command::Update));
    }

    #[test]
    fn run_with_dispatches_migrate_create() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "create", "add_users"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(a) if matches!(a.action, MigrateAction::Create { .. })
        ));
    }

    #[test]
    fn run_with_dispatches_migrate_up() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "up"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(a) if matches!(a.action, MigrateAction::Up)
        ));
    }

    #[test]
    fn run_with_dispatches_migrate_down() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "down"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(a) if matches!(a.action, MigrateAction::Down { .. })
        ));
    }

    #[test]
    fn run_with_dispatches_migrate_status() {
        let cli = Cli::try_parse_from(["ironic", "migrate", "status"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Migrate(a) if matches!(a.action, MigrateAction::Status)
        ));
    }
}
