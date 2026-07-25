use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
    time::Duration,
};

use crate::{CliError, cli::OpenapiArgs};

/// Generates the `OpenAPI` specification by starting the service, fetching
/// `/openapi.json`, and writing it to the specified output file.
///
/// Builds the project first via `cargo build`, starts the binary in the
/// background, polls the `OpenAPI` endpoint until ready, saves the spec, and
/// shuts the service down.
pub(crate) fn execute(args: &OpenapiArgs, output: &mut impl Write) -> Result<(), CliError> {
    build_project(args, output)?;
    start_and_generate(args, output)
}

fn build_project(args: &OpenapiArgs, output: &mut impl Write) -> Result<(), CliError> {
    writeln!(output, "🔨 Building project...").map_err(io_err)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if let Some(pkg) = &args.package {
        cmd.arg("-p");
        cmd.arg(pkg);
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd.status().map_err(|e| CliError::CommandFailed {
        program: "cargo build".into(),
        status: e.to_string(),
    })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            program: "cargo build".into(),
            status: format!("exit code {status}"),
        });
    }
    Ok(())
}

fn start_and_generate(args: &OpenapiArgs, output: &mut impl Write) -> Result<(), CliError> {
    writeln!(output, "🚀 Starting service on port {}...", args.port).map_err(io_err)?;

    let mut child = start_service(args, output)?;
    let url = format!("http://localhost:{}/openapi.json", args.port);

    let spec = wait_for_spec(&url, args.timeout, output)?;

    write_spec(&args.output, &spec, output)?;

    writeln!(output, "✅ OpenAPI spec written to `{}`", args.output).map_err(io_err)?;

    kill_service(&mut child, output)?;
    Ok(())
}

fn start_service(args: &OpenapiArgs, output: &mut impl Write) -> Result<Child, CliError> {
    let _ = output;
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    if let Some(pkg) = &args.package {
        cmd.arg("-p");
        cmd.arg(pkg);
    }
    cmd.env("SERVER_PORT", args.port.to_string());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    cmd.spawn().map_err(|e| CliError::CommandFailed {
        program: "cargo run".into(),
        status: e.to_string(),
    })
}

fn wait_for_spec(
    url: &str,
    timeout_secs: u64,
    output: &mut impl Write,
) -> Result<String, CliError> {
    let deadline = Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > deadline {
            return Err(CliError::CommandFailed {
                program: "openapi".into(),
                status: format!("service did not respond within {timeout_secs} seconds"),
            });
        }

        let agent = ureq::agent();
        let result = agent.get(url).call();
        match result {
            Ok(response) => {
                let mut body = String::new();
                let read_result = response.into_body().as_reader().read_to_string(&mut body);
                if let Err(e) = read_result {
                    return Err(CliError::CommandFailed {
                        program: "openapi".into(),
                        status: format!("failed to read response: {e}"),
                    });
                }
                return Ok(body);
            }
            Err(ureq::Error::StatusCode(code)) => {
                return Err(CliError::CommandFailed {
                    program: "openapi".into(),
                    status: format!("service returned HTTP {code}"),
                });
            }
            Err(_) => {
                writeln!(output, "  waiting for service...").map_err(io_err)?;
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

fn write_spec(path: &str, content: &str, _output: &mut impl Write) -> Result<String, CliError> {
    let formatted = serde_json::from_str::<serde_json::Value>(content)
        .map_err(|e| CliError::CommandFailed {
            program: "openapi".into(),
            status: format!("invalid JSON from spec endpoint: {e}"),
        })
        .and_then(|v| {
            serde_json::to_string_pretty(&v).map_err(|e| CliError::CommandFailed {
                program: "openapi".into(),
                status: format!("failed to format JSON: {e}"),
            })
        })?;

    std::fs::write(path, &formatted).map_err(|e| CliError::io("write", path, e))?;
    Ok(path.to_string())
}

fn kill_service(child: &mut Child, output: &mut impl Write) -> Result<(), CliError> {
    writeln!(output, "  shutting down service...").map_err(io_err)?;
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

fn io_err(error: std::io::Error) -> CliError {
    CliError::io("write output", "stdout", error)
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::cli::OpenapiArgs;

    #[test]
    fn openapi_args_defaults() {
        let args = OpenapiArgs {
            package: None,
            output: "openapi.json".into(),
            port: 8080,
            timeout: 15,
        };
        assert_eq!(args.output, "openapi.json");
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn openapi_args_custom() {
        let args = OpenapiArgs {
            package: Some("auth-service".into()),
            output: "spec.json".into(),
            port: 8081,
            timeout: 30,
        };
        assert_eq!(args.package, Some("auth-service".into()));
        assert_eq!(args.output, "spec.json");
        assert_eq!(args.port, 8081);
    }

    #[test]
    fn io_err_wraps_io_error() {
        let io = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        let err = super::io_err(io);
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_IO"));
    }
}
