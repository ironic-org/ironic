mod cargo;
#[cfg(feature = "hot-reload")]
mod dev;
mod doctor;
mod generate;
mod inspect;
mod migrate;
mod new;
mod update;

use std::io::Write;

use crate::{
    CliError,
    cli::{Cli, Command},
};

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
        Command::Workspace(arguments) => inspect::workspace(&arguments.path, output),
        Command::Routes(arguments) => inspect::routes(&arguments.path, output),
        Command::Graph(arguments) => inspect::graph(&arguments.path, output),
        Command::Migrate(arguments) => migrate::execute(arguments.action, output),
    }
}
