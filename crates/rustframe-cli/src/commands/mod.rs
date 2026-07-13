mod cargo;
mod doctor;
mod generate;
mod new;

use std::io::Write;

use crate::{
    CliError,
    cli::{Cli, Command},
};

pub(crate) fn execute(command: Cli, output: &mut impl Write) -> Result<(), CliError> {
    match command.command {
        Command::New(arguments) => new::execute(&arguments, output),
        Command::Start(arguments) => cargo::execute("run", arguments),
        Command::Build(arguments) => cargo::execute("build", arguments),
        Command::Test(arguments) => cargo::execute("test", arguments),
        Command::Generate(arguments) => generate::execute(arguments, output),
        Command::Doctor => doctor::execute(output),
    }
}
