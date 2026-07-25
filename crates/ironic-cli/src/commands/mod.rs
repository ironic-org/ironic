mod cargo;
#[cfg(feature = "hot-reload")]
mod dev;
mod doctor;
mod generate;
mod inspect;
mod migrate;
mod new;
mod uninstall;
mod update;

use std::io::Write;

use crate::{
    CliError,
    cli::{Cli, Command},
};

/// Routes a parsed command to its handler.
///
/// # Errors
///
/// Returns [`CliError`] when the underlying command fails.
pub(crate) fn execute(command: Cli, output: &mut impl Write) -> Result<(), CliError> {
    match command.command {
        Command::New(arguments) => new::execute(&arguments, output),
        Command::Start(arguments) => cargo::execute("run", &arguments),
        #[cfg(feature = "hot-reload")]
        Command::Dev(arguments) => dev::execute(&arguments, output),
        #[cfg(not(feature = "hot-reload"))]
        Command::Dev(_) => Err(CliError::CommandFailed {
            program: "ironic dev".into(),
            status: "The `dev` command requires the `hot-reload` feature (add `ironic = { features = [\"hot-reload\"] }` to Cargo.toml)".into(),
        }),
        Command::Build(arguments) => cargo::execute("build", &arguments),
        Command::Test(arguments) => cargo::execute("test", &arguments),
        Command::Generate(arguments) => generate::execute(arguments, output),
        Command::Doctor => doctor::execute(output),
        Command::Update => update::execute(output),
        Command::Uninstall => uninstall::execute(output),
        Command::Workspace(arguments) => inspect::workspace(&arguments.path, output),
        Command::Routes(arguments) => inspect::routes(&arguments.path, output),
        Command::Graph(arguments) => inspect::graph(&arguments.path, output),
        Command::Migrate(arguments) => migrate::execute(arguments.action, output),
        Command::Run(arguments) => run_script(&arguments.name, &arguments.args, output),
    }
}

/// Looks up and executes a script defined in `[package.metadata.ironic.scripts]`.
///
/// Scripts are shell commands read from the project's `Cargo.toml` and run via `sh -c`.
fn run_script(name: &str, _args: &[String], output: &mut impl Write) -> Result<(), CliError> {
    let manifest_path = std::env::current_dir()
        .map_err(|e| CliError::io("read current directory", ".", e))?
        .join("Cargo.toml");

    let contents = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CliError::io("read Cargo.toml", &manifest_path, e))?;

    // Simple TOML section parser: find [package.metadata.ironic.scripts]
    let script_section = contents
        .lines()
        .skip_while(|line| !line.trim().starts_with("[package.metadata.ironic.scripts]"))
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .collect::<Vec<_>>()
        .join("\n");

    let target = format!("{name} = ");
    let script = script_section
        .lines()
        .find_map(|line| line.trim().strip_prefix(&target))
        .map(|s| s.trim().trim_matches('"').to_string())
        .ok_or_else(|| CliError::InvalidName {
            name: format!("script `{name}` not found in [package.metadata.ironic.scripts]"),
        })?;

    writeln!(output, "running script `{name}`: {script}")
        .map_err(|e| CliError::io("write output", "stdout", e))?;

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .status()
        .map_err(|e| CliError::io("execute script", "sh", e))?;

    if !status.success() {
        return Err(CliError::InvalidName {
            name: format!("script `{name}` exited with {status}"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cli::*;

    fn run_cmd(command: Command) -> Vec<u8> {
        let cli = Cli { command };
        let mut buf = Vec::new();
        let _ = super::execute(cli, &mut buf);
        buf
    }

    #[test]
    #[ignore = "requires stdin input"]
    fn uninstall_starts_with_instructions() {
        let buf = run_cmd(Command::Uninstall);
        // Should write instructions before waiting for stdin
        assert!(!buf.is_empty());
    }

    #[test]
    fn doctor_runs_without_panicking() {
        let cmd = Command::Doctor;
        let mut buf = Vec::new();
        let result = super::execute(Cli { command: cmd }, &mut buf);
        // Doctor may fail if rustc isn't installed, but shouldn't panic
        assert!(!buf.is_empty() || result.is_err());
    }

    #[test]
    fn workspace_inspects_directory() {
        let cmd = Command::Workspace(InspectArgs { path: ".".into() });
        let mut buf = Vec::new();
        let result = super::execute(Cli { command: cmd }, &mut buf);
        // May fail if Cargo.toml not found, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn routes_inspects_directory() {
        let cmd = Command::Routes(InspectArgs { path: ".".into() });
        let mut buf = Vec::new();
        let result = super::execute(Cli { command: cmd }, &mut buf);
        let _ = result;
    }
}
