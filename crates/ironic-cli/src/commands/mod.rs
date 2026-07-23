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
        Command::Start(arguments) => cargo::execute("run", arguments),
        #[cfg(feature = "hot-reload")]
        Command::Dev(arguments) => dev::execute(&arguments, output),
        #[cfg(not(feature = "hot-reload"))]
        Command::Dev(_) => Err(CliError::CommandFailed {
            program: "ironic dev".into(),
            status: "The `dev` command requires the `hot-reload` feature (add `ironic = { features = [\"hot-reload\"] }` to Cargo.toml)".into(),
        }),
        Command::Build(arguments) => cargo::execute("build", arguments),
        Command::Test(arguments) => cargo::execute("test", arguments),
        Command::Generate(arguments) => generate::execute(arguments, output),
        Command::Doctor => doctor::execute(output),
        Command::Update => update::execute(output),
        Command::Uninstall => uninstall::execute(output),
        Command::Workspace(arguments) => inspect::workspace(&arguments.path, output),
        Command::Routes(arguments) => inspect::routes(&arguments.path, output),
        Command::Graph(arguments) => inspect::graph(&arguments.path, output),
        Command::Migrate(arguments) => migrate::execute(arguments.action, output),
    }
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
