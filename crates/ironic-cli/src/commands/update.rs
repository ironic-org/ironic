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
