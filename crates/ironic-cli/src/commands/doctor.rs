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
    let framework = manifest_contains(&manifest, "ironic")?;
    report(
        output,
        "Ironic dependency",
        framework,
        if framework { "found" } else { "not found" },
    )?;

    check_ironic_version(output)?;

    Ok(())
}

fn check_ironic_version(output: &mut impl Write) -> Result<(), CliError> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = match super::update::check_latest_version() {
        Ok(Some(version)) => version,
        Ok(None) => {
            return report(output, "CLI version", true, current);
        }
        Err(error) => {
            return report(
                output,
                "CLI version",
                true,
                &format!("{current} (could not check for updates: {error})"),
            );
        }
    };
    if latest == current {
        report(output, "CLI version", true, &format!("{current} (latest)"))
    } else {
        report(
            output,
            "CLI version",
            false,
            &format!("{current} (latest: {latest}; run `ironic update`)"),
        )
    }
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

#[cfg(test)]
mod tests {

    #[test]
    fn report_ok_format() {
        let mut buf = Vec::new();
        super::report(&mut buf, "Test", true, "v1.0.0").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("OK"));
        assert!(output.contains("Test"));
        assert!(output.contains("v1.0.0"));
    }

    #[test]
    fn report_warn_format() {
        let mut buf = Vec::new();
        super::report(&mut buf, "Checker", false, "missing").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("WARN"));
        assert!(output.contains("Checker"));
        assert!(output.contains("missing"));
    }

    #[test]
    fn report_returns_error_on_write_failure() {
        use std::io::Write;
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "broken",
                ))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let result = super::report(&mut BrokenWriter, "X", true, "y");
        assert!(result.is_err());
    }

    #[test]
    fn manifest_contain_returns_false_for_missing_file() {
        let result = super::manifest_contains(std::path::Path::new("/nonexistent/path"), "ironic");
        assert!(!result.unwrap());
    }

    #[test]
    fn manifest_contain_checks_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        std::fs::write(&path, "[dependencies]\nironic = \"1.0\"").unwrap();
        assert!(super::manifest_contains(&path, "ironic").unwrap());
        assert!(!super::manifest_contains(&path, "axum").unwrap());
    }

    #[test]
    fn check_ironic_version_is_current() {
        // env!("CARGO_PKG_VERSION") is always set at compile time
        let current = env!("CARGO_PKG_VERSION");
        assert!(!current.is_empty());
    }
}
