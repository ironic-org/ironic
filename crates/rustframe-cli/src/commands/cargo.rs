use std::process::Command;

use crate::{CliError, cli::CargoArgs};

pub(crate) fn execute(subcommand: &str, arguments: CargoArgs) -> Result<(), CliError> {
    let status = Command::new("cargo")
        .arg(subcommand)
        .args(arguments.cargo_args)
        .status()
        .map_err(|error| CliError::io("execute", "cargo", error))?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::CommandFailed {
            program: format!("cargo {subcommand}"),
            status: status.to_string(),
        })
    }
}
