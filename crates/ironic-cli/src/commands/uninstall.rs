use std::io::{BufRead, Write};
use std::process::Command;

use crate::CliError;

/// Known directories created by Ironic to check for cleanup.
const CACHE_DIRS: &[&str] = &[];

pub(crate) fn execute(output: &mut impl Write) -> Result<(), CliError> {
    let map = |e: std::io::Error| CliError::io("write output", "stdout", e);

    writeln!(output, "This will remove Ironic from your system:").map_err(&map)?;
    writeln!(output, "  • Binary: ~/.cargo/bin/ironic").map_err(&map)?;

    if !CACHE_DIRS.is_empty() {
        writeln!(output, "  • Data directories:").map_err(&map)?;
        for dir in CACHE_DIRS {
            writeln!(output, "    - {dir}").map_err(&map)?;
        }
    }

    writeln!(output).map_err(&map)?;
    write!(output, "Continue? (y/N): ").map_err(&map)?;
    output.flush().map_err(&map)?;

    let input = std::io::stdin()
        .lock()
        .lines()
        .next()
        .transpose()
        .map_err(|e| CliError::io("read confirmation", "stdin", e))?
        .unwrap_or_default();

    if !input.trim().eq_ignore_ascii_case("y") {
        writeln!(output, "Aborted.").map_err(&map)?;
        return Ok(());
    }

    // Remove the binary via cargo.
    let status = Command::new("cargo")
        .args(["uninstall", "ironic"])
        .status()
        .map_err(|e| CliError::io("run cargo uninstall", "cargo", e))?;

    if !status.success() {
        writeln!(
            output,
            "  ✗ `cargo uninstall ironic` failed.\n  Run manually: cargo uninstall ironic"
        )
        .map_err(&map)?;
        return Err(CliError::CommandFailed {
            program: "cargo uninstall ironic".into(),
            status: "non-zero exit status".into(),
        });
    }

    writeln!(output, "  ✓ Ironic has been uninstalled.").map_err(&map)?;
    Ok(())
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use crate::CliError;

    #[test]
    fn cache_dirs_currently_empty() {
        assert!(super::CACHE_DIRS.is_empty());
    }

    #[test]
    fn execute_writes_prompt() {
        let mut buf = Vec::new();
        // execute will try to read from stdin when it shows the prompt,
        // but we don't actually provide input. It should still write the prompt.
        let result = super::execute(&mut buf);
        let output = String::from_utf8(buf).unwrap_or_default();
        // At minimum the prompt text should be there
        assert!(output.contains("remove Ironic") || output.contains("Continue"));
        // If stdin has no input, it should abort
        if result.is_ok() {
            assert!(output.contains("Aborted"));
        }
    }

    #[test]
    fn uninstall_output_format() {
        let mut buf = Vec::new();
        let result = super::execute(&mut buf);
        let output = String::from_utf8(buf).unwrap_or_default();
        if let Err(CliError::CommandFailed { program, .. }) = &result {
            assert!(program.contains("cargo uninstall"));
        }
        // Should either abort or try uninstall
        assert!(
            output.contains("remove") || output.contains("uninstall") || output.contains("Aborted")
        );
    }
}
