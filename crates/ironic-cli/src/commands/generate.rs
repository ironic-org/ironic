use std::{env, io::Write};

use crate::{
    CliError,
    cli::{GenerateArgs, Generator, ReadyResourceVariant},
    generators,
};

/// Executes the selected generator and prints a summary to `output`.
///
/// # Errors
///
/// Returns [`CliError`] for invalid names, I/O failures, or file conflicts.
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

#[cfg(test)]
mod tests {
    use crate::cli::{GenerateArgs, Generator, NameArgs};
    use crate::CliError;

    #[ignore]
    #[test]
    fn execute_module_generator_fails_in_non_project_dir() {
        let args = GenerateArgs {
            generator: Generator::Module(NameArgs { name: "test_mod".into() }),
        };
        let mut buf = Vec::new();
        let result = super::execute(args, &mut buf);
        assert!(result.is_err());
    }

    #[ignore]
    #[test]
    fn execute_controller_generator_fails_in_non_project_dir() {
        let args = GenerateArgs {
            generator: Generator::Controller(NameArgs { name: "my_ctrl".into() }),
        };
        let mut buf = Vec::new();
        let result = super::execute(args, &mut buf);
        assert!(result.is_err());
    }

    #[ignore]
    #[test]
    fn execute_service_generator_fails_in_non_project_dir() {
        let args = GenerateArgs {
            generator: Generator::Service(NameArgs { name: "my_svc".into() }),
        };
        let mut buf = Vec::new();
        let result = super::execute(args, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn execute_bad_name_returns_invalid_name() {
        let args = GenerateArgs {
            generator: Generator::Module(NameArgs { name: "123".into() }),
        };
        let mut buf = Vec::new();
        let result = super::execute(args, &mut buf);
        assert!(matches!(result, Err(CliError::InvalidName { .. })));
    }

    #[test]
    fn execute_keyword_name_returns_invalid_name() {
        let args = GenerateArgs {
            generator: Generator::Module(NameArgs { name: "mod".into() }),
        };
        let mut buf = Vec::new();
        let result = super::execute(args, &mut buf);
        assert!(matches!(result, Err(CliError::InvalidName { .. })));
    }

    #[ignore]
    #[test]
    fn execute_single_file_generators_fail_in_non_project_dir() {
        for variant in ["decorator", "filter", "gateway", "guard", "interceptor", "middleware", "pipe", "provider"] {
            let generator = match variant {
                "decorator" => Generator::Decorator(NameArgs { name: "test".into() }),
                "filter" => Generator::Filter(NameArgs { name: "test".into() }),
                "gateway" => Generator::Gateway(NameArgs { name: "test".into() }),
                "guard" => Generator::Guard(NameArgs { name: "test".into() }),
                "interceptor" => Generator::Interceptor(NameArgs { name: "test".into() }),
                "middleware" => Generator::Middleware(NameArgs { name: "test".into() }),
                "pipe" => Generator::Pipe(NameArgs { name: "test".into() }),
                "provider" => Generator::Provider(NameArgs { name: "test".into() }),
                _ => unreachable!(),
            };
            let args = GenerateArgs { generator: generator };
            let mut buf = Vec::new();
            let result = super::execute(args, &mut buf);
            assert!(result.is_err(), "{variant} should fail without a src/ directory");
        }
    }
}
