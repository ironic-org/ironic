use std::{env, io::Write};

use crate::{
    CliError,
    cli::{GenerateArgs, Generator},
    generators,
};

pub(crate) fn execute(arguments: GenerateArgs, output: &mut impl Write) -> Result<(), CliError> {
    let root =
        env::current_dir().map_err(|error| CliError::io("read current directory", ".", error))?;
    let report = match arguments.generator {
        Generator::Module(arguments) => generators::generate_module(&root, &arguments.name),
        Generator::Controller(arguments) => generators::generate_controller(&root, &arguments.name),
        Generator::Service(arguments) => generators::generate_service(&root, &arguments.name),
        Generator::Resource(arguments) => generators::generate_resource(&root, &arguments.name),
    }?;

    for path in report.created {
        writeln!(output, "created {}", path.display())
            .map_err(|error| CliError::io("write output", "stdout", error))?;
    }
    for path in report.unchanged {
        writeln!(output, "unchanged {}", path.display())
            .map_err(|error| CliError::io("write output", "stdout", error))?;
    }
    for instruction in report.manual_instructions {
        writeln!(output, "manual: {instruction}")
            .map_err(|error| CliError::io("write output", "stdout", error))?;
    }
    Ok(())
}
