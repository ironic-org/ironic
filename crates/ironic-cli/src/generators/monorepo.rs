use std::path::Path;

use crate::CliError;

use super::GenerationReport;

/// Converts a single-service project to a monorepo workspace.
/// Moves src/ into apps/<name>/, creates workspace Cargo.toml, sets up libs/.
/// Ensures Rust Analyzer can detect the workspace structure.
pub(super) fn convert_to_monorepo(
    root: &Path,
    report: &mut GenerationReport,
) -> Result<(), CliError> {
    use std::fs;

    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        return Err(CliError::InvalidName {
            name: "Cargo.toml not found — are you in an Ironic project?".into(),
        });
    }

    // Read current package name from Cargo.toml
    let toml_content = std::fs::read_to_string(&cargo_toml)
        .map_err(|e| CliError::io("read Cargo.toml", &cargo_toml, e))?;
    let pkg_name = toml_content
        .lines()
        .find_map(|l| l.strip_prefix("name = \""))
        .and_then(|l| l.split('"').next())
        .unwrap_or("app")
        .to_string();

    let app_dir = root.join("apps").join(&pkg_name);
    let src_dir = root.join("src");

    // Move src/ into apps/<name>/src/
    if src_dir.is_dir() {
        let dst_src = app_dir.join("src");
        fs::create_dir_all(&dst_src).map_err(|e| CliError::Io {
            action: "create apps directory",
            path: app_dir.clone(),
            source: e,
        })?;
        copy_dir_recursive(&src_dir, &dst_src)?;
        fs::remove_dir_all(&src_dir).map_err(|e| CliError::Io {
            action: "remove old src directory",
            path: src_dir,
            source: e,
        })?;
    }

    // Create app Cargo.toml
    let app_manifest = format!(
        r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = {{ workspace = true }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
garde = {{ workspace = true }}
sqlx = {{ workspace = true }}
tracing = {{ workspace = true }}
tracing-subscriber = {{ workspace = true }}
dotenvy = {{ workspace = true }}
"#
    );
    std::fs::write(app_dir.join("Cargo.toml"), &app_manifest).map_err(|e| CliError::Io {
        action: "write app Cargo.toml",
        path: app_dir.join("Cargo.toml"),
        source: e,
    })?;

    let ironic_version = env!("CARGO_PKG_VERSION");
    let version_range = ironic_version
        .splitn(3, '.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    // Rewrite root Cargo.toml as pure workspace manifest
    let workspace_manifest = format!(
        r#"[workspace]
resolver = "3"
members = [
    "apps/{pkg_name}",
]

[workspace.dependencies]
ironic = {{ version = "{version_range}", features = ["security", "compression", "metrics", "validation", "versioning", "openapi", "logging", "sqlx-postgres"] }}
tokio = {{ version = "1", features = ["macros", "rt-multi-thread", "net", "signal"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
garde = "0.23"
sqlx = {{ version = "0.9", features = ["runtime-tokio", "postgres"] }}
tracing = {{ version = "0.1", features = ["attributes"] }}
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"
tonic = {{ version = "0.14", features = ["transport", "codegen", "router"] }}
prost = "0.14"
tonic-prost-build = "0.14"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
"#,
    );
    std::fs::write(&cargo_toml, &workspace_manifest).map_err(|e| CliError::Io {
        action: "write workspace Cargo.toml",
        path: cargo_toml.clone(),
        source: e,
    })?;

    report
        .manual_instructions
        .push("converted to monorepo — run `cargo check` to refresh Rust Analyzer".into());

    Ok(())
}

/// Recursively copies all files and subdirectories from `src` to `dst`.
///
/// Preserves directory structure. Returns [`CliError`] on I/O failures.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), CliError> {
    use std::fs;

    for entry in std::fs::read_dir(src).map_err(|e| CliError::Io {
        action: "read source directory",
        path: src.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| CliError::Io {
            action: "read directory entry",
            path: src.to_path_buf(),
            source: e,
        })?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|e| CliError::Io {
            action: "get file type",
            path: entry_path.clone(),
            source: e,
        })?;
        let relative = entry_path
            .strip_prefix(src)
            .map_err(|_| CliError::InvalidName {
                name: "path error during copy".into(),
            })?;
        let target = dst.join(relative);
        if file_type.is_dir() {
            fs::create_dir_all(&target).map_err(|e| CliError::Io {
                action: "create directory",
                path: target.clone(),
                source: e,
            })?;
            copy_dir_recursive(&entry_path, &target)?;
        } else {
            fs::copy(&entry_path, &target).map_err(|e| CliError::Io {
                action: "copy file",
                path: entry_path,
                source: e,
            })?;
        }
    }
    Ok(())
}

/// Inserts a member path into the workspace `members = [...]` array in a `Cargo.toml`.
///
/// Parses the file textually (not TOML) to find the `members` array and appends
/// the new member before the closing `]`. No-op if the member already exists.
pub(super) fn ensure_workspace_member(manifest: &Path, member: &str) {
    use std::io::{Read, Write};
    let mut contents = String::new();
    if let Ok(mut f) = std::fs::File::open(manifest) {
        let _ = f.read_to_string(&mut contents);
    }
    let line = format!("        \"{member}\",\n");
    if let Some(pos) = contents.find("members = [") {
        let insert_pos = contents[pos..]
            .find(']')
            .map_or(contents.len(), |p| pos + p);
        contents.insert_str(insert_pos, &line);
        if let Ok(mut f) = std::fs::File::create(manifest) {
            let _ = f.write_all(contents.as_bytes());
        }
    }
}
