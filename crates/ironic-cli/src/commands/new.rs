use std::{env, io::Write};

use crate::{CliError, cli::NewArgs, generators::project};

pub(crate) fn execute(arguments: &NewArgs, output: &mut impl Write) -> Result<(), CliError> {
    let current =
        env::current_dir().map_err(|error| CliError::io("read current directory", ".", error))?;
    let use_current = matches!(arguments.name.as_str(), "." | "./");
    let (destination, project_name) = if use_current {
        let project_name = project::name_from_directory(&current)?;
        (current, project_name)
    } else {
        let project_name = project::directory_name(&arguments.name)?;
        (current.join(&project_name), project_name)
    };
    let report = project::create(
        &destination,
        &project_name,
        arguments.framework_workspace.as_deref(),
    )?;
    writeln!(output, "Created `{}`", report.destination.display())
        .map_err(|error| CliError::io("write output", "stdout", error))?;
    let next = if use_current {
        "ironic start".to_owned()
    } else {
        format!("cd {project_name} && ironic start")
    };
    writeln!(output, "Next: {next}")
        .map_err(|error| CliError::io("write output", "stdout", error))?;
    Ok(())
}
