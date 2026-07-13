use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// The Ironic command-line interface.
#[derive(Debug, Parser)]
#[command(name = "ironic", version, about)]
pub struct Cli {
    /// Command to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported MVP commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Creates a new application project.
    New(NewArgs),
    /// Runs the current project through Cargo.
    Start(CargoArgs),
    /// Builds the current project through Cargo.
    Build(CargoArgs),
    /// Tests the current project through Cargo.
    Test(CargoArgs),
    /// Generates application source files.
    #[command(alias = "g")]
    Generate(GenerateArgs),
    /// Checks the local Rust and project environment.
    Doctor,
}

/// Arguments for project creation.
#[derive(Debug, Args)]
pub struct NewArgs {
    /// Project name and destination directory.
    pub name: String,
    /// Uses local framework crates from a workspace checkout.
    #[arg(long, hide = true)]
    pub framework_workspace: Option<PathBuf>,
}

/// Arguments passed through to Cargo after `--`.
#[derive(Debug, Args)]
pub struct CargoArgs {
    /// Additional Cargo arguments.
    #[arg(last = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<String>,
}

/// Generator selection and source name.
#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Artifact to generate.
    #[command(subcommand)]
    pub generator: Generator,
}

/// Supported source generators.
#[derive(Debug, Subcommand)]
pub enum Generator {
    /// Generates a module.
    #[command(alias = "mo")]
    Module(NameArgs),
    /// Generates a controller.
    #[command(alias = "co")]
    Controller(NameArgs),
    /// Generates a service.
    #[command(alias = "s")]
    Service(NameArgs),
    /// Generates a module, service, and controller vertical slice.
    #[command(alias = "res")]
    Resource(NameArgs),
}

/// A named generator target.
#[derive(Debug, Args)]
pub struct NameArgs {
    /// Resource or module name.
    pub name: String,
}
