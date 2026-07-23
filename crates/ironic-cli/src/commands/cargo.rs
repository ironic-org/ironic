use std::process::Command;

use crate::{CliError, cli::CargoArgs};

/// Runs `cargo <subcommand>` with the given arguments.
///
/// # Errors
///
/// Returns [`CliError::Io`] if the Cargo binary cannot be launched.
/// Returns [`CliError::CommandFailed`] if Cargo exits with a non-zero status.
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

#[cfg(test)]
mod tests {
    use crate::cli::CargoArgs;
    use crate::CliError;

    #[test]
    fn cargo_args_construction() {
        let args = CargoArgs { cargo_args: vec!["--release".into(), "--features".into(), "foo".into()] };
        assert_eq!(args.cargo_args.len(), 3);
        assert_eq!(args.cargo_args[0], "--release");
    }

    #[test]
    fn cargo_args_empty_by_default() {
        let args = CargoArgs { cargo_args: vec![] };
        assert!(args.cargo_args.is_empty());
    }

    #[test]
    fn execute_with_nonexistent_subcommand() {
        let result = super::execute("nonexistent_subcommand_xyz", CargoArgs { cargo_args: vec![] });
        // Cargo binary exists, subcommand fails -> CommandFailed
        assert!(result.is_err());
        if let Err(CliError::CommandFailed { program, .. }) = result {
            assert!(program.contains("nonexistent_subcommand_xyz"));
        } else {
            panic!("expected CommandFailed");
        }
    }
}
