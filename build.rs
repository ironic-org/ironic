#![allow(missing_docs)]

fn main() {
    let git_sha = std::env::var("GIT_SHA")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "unknown".to_string())
        });

    let build_timestamp = std::env::var("BUILD_TIMESTAMP")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("{now}")
        });

    let rust_version = std::env::var("RUSTC_VERSION")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| rustc_version().unwrap_or_else(|| "unknown".to_string()));

    println!("cargo::rustc-env=IRONIC_GIT_SHA={git_sha}");
    println!("cargo::rustc-env=IRONIC_BUILD_TIMESTAMP={build_timestamp}");
    println!("cargo::rustc-env=IRONIC_RUST_VERSION={rust_version}");
}

fn rustc_version() -> Option<String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}
