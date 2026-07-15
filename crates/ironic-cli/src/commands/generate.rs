use std::{env, io::Write};

use crate::{
    CliError,
    cli::{GenerateArgs, Generator, ReadyResourceVariant},
    generators,
};

pub(crate) fn execute(arguments: GenerateArgs, output: &mut impl Write) -> Result<(), CliError> {
    let root =
        env::current_dir().map_err(|error| CliError::io("read current directory", ".", error))?;
    let report = match arguments.generator {
        Generator::Module(arguments) => generators::generate_module(&root, &arguments.name),
        Generator::Controller(arguments) => generators::generate_controller(&root, &arguments.name),
        Generator::Service(arguments) => generators::generate_service(&root, &arguments.name),
        Generator::Repository(arguments) => generators::generate_repository(&root, &arguments.name),
        Generator::Resource(arguments) => generators::generate_resource(&root, &arguments.name),
        Generator::Decorator(arguments) => generators::generate_decorator(&root, &arguments.name),
        Generator::Filter(arguments) => generators::generate_filter(&root, &arguments.name),
        Generator::Gateway(arguments) => generators::generate_gateway(&root, &arguments.name),
        Generator::Guard(arguments) => generators::generate_guard(&root, &arguments.name),
        Generator::Interceptor(arguments) => {
            generators::generate_interceptor(&root, &arguments.name)
        }
        Generator::Middleware(arguments) => generators::generate_middleware(&root, &arguments.name),
        Generator::Pipe(arguments) => generators::generate_pipe(&root, &arguments.name),
        Generator::Provider(arguments) => generators::generate_provider(&root, &arguments.name),
        Generator::ReadyResource(arguments) => match arguments.variant {
            ReadyResourceVariant::Auth => generators::generate_ready_resource(&root, "auth"),
            ReadyResourceVariant::AuthBasic => generators::generate_ready_resource_basic(&root),
            ReadyResourceVariant::AuthJwt => generators::generate_ready_resource_jwt(&root),
            ReadyResourceVariant::AuthOauth => generators::generate_ready_resource_oauth(&root),
            ReadyResourceVariant::FileUpload => {
                generators::generate_ready_resource_file_upload(&root)
            }
            ReadyResourceVariant::Email => generators::generate_ready_resource_email(&root),
        },
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
