mod cargo;
mod dev;
mod doctor;
mod generate;
mod inspect;
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
        Command::Dev(arguments) => dev::execute(&arguments, output),
        Command::Build(arguments) => cargo::execute("build", arguments),
        Command::Test(arguments) => cargo::execute("test", arguments),
        Command::Generate(arguments) => generate::execute(arguments, output),
        Command::Doctor => doctor::execute(output),
        Command::Update => update::execute(output),
        Command::Routes(arguments) => inspect::routes(&arguments.path, output),
        Command::Graph(arguments) => inspect::graph(&arguments.path, output),
    }
}
