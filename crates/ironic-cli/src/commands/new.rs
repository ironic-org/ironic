use std::{env, io::Write};

use crate::{CliError, cli::NewArgs, generators::project};

/// Creates a new Ironic project from `name` at the resolved destination.
///
/// # Errors
///
/// Returns [`CliError::InvalidName`] if `name` cannot form a safe identifier.
/// Returns [`CliError::Io`] if filesystem inspection or creation fails.
/// Returns [`CliError::FileConflict`] if the destination contains conflicting sources.
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

#[cfg(test)]
mod tests {
    use crate::cli::NewArgs;

    #[test]
    fn execute_with_bad_name_returns_error() {
        let args = NewArgs {
            name: "123".into(),
            framework_workspace: None,
        };
        let mut buf = Vec::new();
        let result = super::execute(&args, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn execute_with_keyword_name_returns_error() {
        let args = NewArgs {
            name: "mod".into(),
            framework_workspace: None,
        };
        let mut buf = Vec::new();
        let result = super::execute(&args, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn execute_with_current_dir_is_context_dependent() {
        let args = NewArgs {
            name: ".".into(),
            framework_workspace: None,
        };
        let mut buf = Vec::new();
        let result = super::execute(&args, &mut buf);
        // This may succeed if cwd is empty, or fail if cwd has files
        // Either way it shouldn't panic
        let _ = result;
    }

    #[test]
    fn new_args_debug() {
        let args = NewArgs {
            name: "test".into(),
            framework_workspace: None,
        };
        let debug = format!("{args:?}");
        assert!(debug.contains("test"));
    }
}
