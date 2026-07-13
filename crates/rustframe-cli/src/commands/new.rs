use std::{env, io::Write};

use crate::{CliError, cli::NewArgs, generators::project};

pub(crate) fn execute(arguments: &NewArgs, output: &mut impl Write) -> Result<(), CliError> {
    let directory = project::directory_name(&arguments.name)?;
    let destination = env::current_dir()
        .map_err(|error| CliError::io("read current directory", ".", error))?
        .join(&directory);
    let report = project::create(
        &destination,
        &arguments.name,
        arguments.framework_workspace.as_deref(),
    )?;
    writeln!(output, "Created `{}`", report.destination.display())
        .map_err(|error| CliError::io("write output", "stdout", error))?;
    writeln!(output, "Next: cd {directory} && cargo run")
        .map_err(|error| CliError::io("write output", "stdout", error))?;
    Ok(())
}
