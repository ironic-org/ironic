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
pub fn run() -> Result<(), CliError> {
    use clap::Parser;

    run_with(cli::Cli::parse(), &mut io::stdout())
}

/// Executes an already-parsed command and writes user-facing output to `output`.
///
/// # Errors
///
/// Returns [`CliError`] when the selected command fails.
pub fn run_with(command: cli::Cli, output: &mut impl Write) -> Result<(), CliError> {
    commands::execute(command, output)
}
