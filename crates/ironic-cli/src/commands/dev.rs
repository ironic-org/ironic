use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[cfg(feature = "hot-reload")]
use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::CliError;

#[allow(clippy::too_many_lines)]
pub(crate) fn execute(
    cargo_args: &crate::cli::CargoArgs,
    output: &mut impl Write,
) -> Result<(), CliError> {
    let root = std::env::current_dir()
        .map_err(|error| CliError::io("read current directory", ".", error))?;

    let (src_dir, package_flag) = resolve_target(&root, cargo_args)?;

    let label = package_flag.as_deref().unwrap_or("src");
    writeln!(output, "ironic dev — watching for changes in {label}/").map_err(io_err)?;
    writeln!(output, "Press Ctrl+C to stop").map_err(io_err)?;

    let running = Arc::new(AtomicBool::new(true));
    let child = Arc::new(Mutex::new(None::<Child>));

    start_process(
        &root,
        package_flag.as_deref(),
        &cargo_args.cargo_args,
        &child,
        output,
    )?;

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|error| CliError::CommandFailed {
        program: "signal handler".into(),
        status: error.to_string(),
    })?;

    let child_clone = child.clone();
    let running_clone = running.clone();
    let root_clone = root.clone();
    let package_flag_clone = package_flag.clone();
    let args_clone = cargo_args.cargo_args.clone();

    let restart = move || {
        kill_child(&child_clone);
        std::thread::sleep(Duration::from_millis(300));
        if running_clone.load(Ordering::SeqCst) {
            let _ = start_process(
                &root_clone,
                package_flag_clone.as_deref(),
                &args_clone,
                &child_clone,
                &mut std::io::stdout(),
            );
        }
    };

    let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
        if let Ok(event) = result {
            let source_changed = event.paths.iter().any(|p| is_rust_file(p));
            if !source_changed {
                return;
            }
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    restart();
                }
                _ => {}
            }
        }
    })
    .map_err(|error| CliError::CommandFailed {
        program: "file watcher".into(),
        status: error.to_string(),
    })?;

    watcher
        .watch(&src_dir, RecursiveMode::Recursive)
        .map_err(|error| CliError::CommandFailed {
            program: "watch".into(),
            status: error.to_string(),
        })?;

    let cargo_toml = resolve_cargo_toml(&root, package_flag.as_deref());
    if cargo_toml.is_file() {
        let _ = watcher.watch(&cargo_toml, RecursiveMode::NonRecursive);
    }
    let ironic_toml = root.join("ironic.toml");
    if ironic_toml.is_file() {
        let _ = watcher.watch(&ironic_toml, RecursiveMode::NonRecursive);
    }

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(200));
    }

    kill_child(&child);

    writeln!(output, "\nironic dev stopped.").map_err(io_err)?;
    Ok(())
}

/// Resolves the source directory and optional package name for dev mode.
///
/// When `-p <name>` is given, looks for `apps/<name>/src/` (monorepo) or
/// `<name>/src/` (adjacent directory). Otherwise uses `src/` in the current
/// directory.
fn resolve_target(
    root: &Path,
    cargo_args: &crate::cli::CargoArgs,
) -> Result<(PathBuf, Option<String>), CliError> {
    if let Some(pkg) = &cargo_args.package {
        let app_root = find_app_root(root, pkg)?;
        let app_src = app_root.join("src");
        if !app_src.is_dir() {
            return Err(CliError::io(
                "read",
                &app_src,
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "app `{pkg}` has no src/ directory at `{}`",
                        app_src.display()
                    ),
                ),
            ));
        }
        Ok((app_src, Some(pkg.clone())))
    } else {
        let local_src = root.join("src");
        if !local_src.is_dir() {
            if root.join("apps").is_dir() {
                return Err(CliError::io(
                    "read",
                    &local_src,
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "src/ directory not found — use `ironic dev -p <app-name>` to run a specific app in this monorepo",
                    ),
                ));
            }
            return Err(CliError::io(
                "read",
                &local_src,
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "src/ directory not found — are you in an Ironic project?",
                ),
            ));
        }
        Ok((local_src, None))
    }
}

/// Finds the app root for a given package name, checking both `apps/<name>/`
/// and `<name>/` paths.
fn find_app_root(root: &Path, pkg: &str) -> Result<PathBuf, CliError> {
    // Check apps/<pkg>/ (monorepo convention) first
    let apps_dir = root.join("apps");
    if apps_dir.is_dir() {
        let app_root = apps_dir.join(pkg);
        if app_root.is_dir() {
            return Ok(app_root);
        }
    }
    // Fallback: <pkg>/ as a cargo workspace member adjacent to root
    let app_root = root.join(pkg);
    if app_root.is_dir() && app_root.join("Cargo.toml").is_file() {
        return Ok(app_root);
    }
    Err(CliError::io(
        "read",
        root.join("apps").join(pkg),
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("app `{pkg}` not found — have you run `ironic generate app {pkg}`?"),
        ),
    ))
}

/// Returns the Cargo.toml path for the target app, or the root Cargo.toml.
fn resolve_cargo_toml(root: &Path, package: Option<&str>) -> PathBuf {
    if let Some(pkg) = package {
        // Try apps/<pkg>/Cargo.toml, then <pkg>/Cargo.toml
        let apps_path = root.join("apps").join(pkg).join("Cargo.toml");
        if apps_path.is_file() {
            return apps_path;
        }
        root.join(pkg).join("Cargo.toml")
    } else {
        root.join("Cargo.toml")
    }
}

fn kill_child(child: &Arc<Mutex<Option<Child>>>) {
    if let Ok(mut guard) = child.lock()
        && let Some(ref mut c) = *guard
    {
        let _ = c.kill();
        let _ = c.wait();
    }
}

fn start_process(
    root: &Path,
    package: Option<&str>,
    cargo_args: &[String],
    child: &Arc<Mutex<Option<Child>>>,
    output: &mut impl Write,
) -> Result<(), CliError> {
    writeln!(output, "\n🔨 Building...").map_err(io_err)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    if let Some(pkg) = package {
        cmd.arg("-p");
        cmd.arg(pkg);
    }
    for arg in cargo_args {
        if arg == "--" {
            continue;
        }
        cmd.arg(arg);
    }
    cmd.current_dir(root);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let spawned = cmd.spawn().map_err(|error| CliError::CommandFailed {
        program: "cargo run".into(),
        status: error.to_string(),
    })?;

    if let Ok(mut guard) = child.lock() {
        *guard = Some(spawned);
    }

    Ok(())
}

fn is_rust_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "rs" || ext == "toml")
}

fn io_err(error: std::io::Error) -> CliError {
    CliError::io("write output", "stdout", error)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::CliError;

    #[test]
    fn is_rust_file_accepts_rs() {
        assert!(super::is_rust_file(Path::new("main.rs")));
    }

    #[test]
    fn is_rust_file_accepts_toml() {
        assert!(super::is_rust_file(Path::new("Cargo.toml")));
    }

    #[test]
    fn is_rust_file_rejects_other_extensions() {
        assert!(!super::is_rust_file(Path::new("main.py")));
        assert!(!super::is_rust_file(Path::new("index.html")));
        assert!(!super::is_rust_file(Path::new("Makefile")));
    }

    #[test]
    fn is_rust_file_rejects_no_extension() {
        assert!(!super::is_rust_file(Path::new("LICENSE")));
        assert!(!super::is_rust_file(Path::new("Makefile")));
    }

    #[test]
    fn is_rust_file_rejects_directory() {
        assert!(!super::is_rust_file(Path::new("src")));
    }

    #[test]
    fn is_rust_file_checks_extension_only() {
        assert!(super::is_rust_file(Path::new("/path/to/module.rs")));
        assert!(super::is_rust_file(Path::new("/path/to/Cargo.toml")));
    }

    #[test]
    fn io_err_wraps_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        let err = super::io_err(io);
        let msg = err.to_string();
        assert!(msg.contains("RF_CLI_IO"));
        assert!(msg.contains("write output"));
        assert!(msg.contains("pipe broken"));
    }

    #[test]
    fn io_err_produces_io_variant() {
        let io = std::io::Error::other("write failed");
        let err = super::io_err(io);
        assert!(matches!(err, CliError::Io { .. }));
    }
}
