use std::{env, io::Write, path::Path, process::Command};

use crate::CliError;

pub(crate) fn execute(output: &mut impl Write) -> Result<(), CliError> {
    check_tool("Rust", "rustc", &["--version"], output)?;
    check_tool("Cargo", "cargo", &["--version"], output)?;

    let root =
        env::current_dir().map_err(|error| CliError::io("read current directory", ".", error))?;
    let manifest = root.join("Cargo.toml");
    report(
        output,
        "Project manifest",
        manifest.is_file(),
        &manifest.display().to_string(),
    )?;
    let framework = manifest_contains(&manifest, "rustframe")?;
    report(
        output,
        "RustFrame dependency",
        framework,
        if framework { "found" } else { "not found" },
    )?;
    Ok(())
}

fn check_tool(
    label: &str,
    program: &str,
    arguments: &[&str],
    output: &mut impl Write,
) -> Result<(), CliError> {
    match Command::new(program).args(arguments).output() {
        Ok(result) if result.status.success() => {
            let detail = String::from_utf8_lossy(&result.stdout);
            report(output, label, true, detail.trim())
        }
        Ok(result) => report(output, label, false, &result.status.to_string()),
        Err(error) => report(output, label, false, &error.to_string()),
    }
}

fn manifest_contains(path: &Path, needle: &str) -> Result<bool, CliError> {
    if !path.is_file() {
        return Ok(false);
    }
    let source =
        std::fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?;
    Ok(source.contains(needle))
}

fn report(
    output: &mut impl Write,
    label: &str,
    success: bool,
    detail: &str,
) -> Result<(), CliError> {
    writeln!(
        output,
        "{label:<22} {} {detail}",
        if success { "OK" } else { "WARN" }
    )
    .map_err(|error| CliError::io("write output", "stdout", error))
}
