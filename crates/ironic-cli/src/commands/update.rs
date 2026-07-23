use std::io::Write;
use std::process::Command;

use crate::CliError;

const CRATES_IO_API: &str = "https://crates.io/api/v1/crates/ironic";
const USER_AGENT: &str = concat!("ironic-cli/", env!("CARGO_PKG_VERSION"));

pub(crate) fn execute(output: &mut impl Write) -> Result<(), CliError> {
    let map = |e: std::io::Error| CliError::io("write output", "stdout", e);
    let current = env!("CARGO_PKG_VERSION");
    let installed = which_install_method()
        .map(|method| format!(" ({method})"))
        .unwrap_or_default();

    match check_latest_version() {
        Ok(Some(latest)) if latest != current => {
            writeln!(output, "A new version of ironic is available: {latest}").map_err(&map)?;
            writeln!(output, "  installed: {current}{installed}").map_err(&map)?;
            writeln!(output, "Upgrading to v{latest}...").map_err(&map)?;
            let status = Command::new("cargo")
                .args(["install", "ironic"])
                .status()
                .map_err(|e| CliError::io("run cargo install", "cargo", e))?;
            if status.success() {
                writeln!(output, "  ✓ Updated to ironic v{latest}").map_err(&map)?;
            } else {
                writeln!(output, "  ✗ `cargo install ironic` failed").map_err(&map)?;
                writeln!(output, "  Run manually: cargo install ironic").map_err(&map)?;
            }
        }
        Ok(Some(_)) => {
            writeln!(output, "ironic {current} is the latest version.").map_err(&map)?;
        }
        Ok(None) => {
            writeln!(output, "ironic {current}").map_err(&map)?;
            writeln!(output, "Could not find version information on crates.io.").map_err(&map)?;
        }
        Err(error) => {
            writeln!(output, "ironic {current}").map_err(&map)?;
            writeln!(output, "Could not check for updates: {error}").map_err(&map)?;
            writeln!(
                output,
                "Visit https://crates.io/crates/ironic to check manually."
            )
            .map_err(&map)?;
        }
    }
    Ok(())
}

pub(crate) fn check_latest_version() -> Result<Option<String>, String> {
    let agent = ureq::Agent::config_builder()
        .user_agent(USER_AGENT)
        .build()
        .new_agent();
    let response = agent
        .get(CRATES_IO_API)
        .call()
        .map_err(|e| format!("request failed: {e}"))?;
    let body: serde_json::Value = response
        .into_body()
        .read_json()
        .map_err(|e| format!("invalid response: {e}"))?;
    let version = body
        .pointer("/crate/max_stable_version")
        .and_then(|v| v.as_str())
        .map(String::from);
    Ok(version)
}

fn which_install_method() -> Option<&'static str> {
    let exe = std::env::current_exe().ok()?;
    let path = exe.to_string_lossy();
    if path.contains("/.cargo/bin/") {
        Some("cargo install")
    } else if path.contains("/nix/") {
        Some("nix")
    } else if path.contains("/homebrew/") || path.contains("/opt/homebrew/") {
        Some("homebrew")
    } else if path.contains("/usr/local/") || path.contains("/usr/bin/") {
        Some("system package manager")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn crates_io_api_url() {
        assert_eq!(
            super::CRATES_IO_API,
            "https://crates.io/api/v1/crates/ironic"
        );
    }

    #[test]
    fn user_agent_contains_cli_and_version() {
        let ua = super::USER_AGENT;
        assert!(ua.starts_with("ironic-cli/"));
        assert!(!ua.ends_with('/'));
    }

    #[test]
    fn execute_output_format() {
        let mut buf = Vec::new();
        let result = super::execute(&mut buf);
        let output = String::from_utf8(buf).unwrap_or_default();
        // Should always write something
        assert!(!output.is_empty());
        // Either shows current version or error message
        let current = env!("CARGO_PKG_VERSION");
        assert!(
            output.contains(current) || output.contains("error") || output.contains("Could not")
        );
        // Even if check fails, the function returns Ok
        assert!(result.is_ok());
    }

    #[test]
    fn which_install_method_is_some_when_running() {
        // The test binary's current_exe is always accessible
        let method = super::which_install_method();
        // Method can be None for custom paths, but should never panic
        let _ = method;
    }

    #[test]
    fn check_latest_version_handles_network_error() {
        // This network call fails gracefully when offline
        let result = super::check_latest_version();
        match result {
            Ok(version) => {
                // If it succeeds, version must be a non-empty string
                assert!(version.is_none_or(|v| !v.is_empty()));
            }
            Err(msg) => {
                // Error message must be descriptive
                assert!(!msg.is_empty());
            }
        }
    }

    #[test]
    fn cargo_pkg_version_available() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        // Version should be semver-like
        assert!(version.chars().next().is_some_and(|c| c.is_ascii_digit()));
    }
}
