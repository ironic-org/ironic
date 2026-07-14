use std::{
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::CliError;

#[allow(clippy::too_many_lines)]
pub(crate) fn execute(
    cargo_args: &crate::cli::CargoArgs,
    output: &mut impl Write,
) -> Result<(), CliError> {
    let root = std::env::current_dir()
        .map_err(|error| CliError::io("read current directory", ".", error))?;
    let src_dir = root.join("src");

    if !src_dir.is_dir() {
        return Err(CliError::io(
            "read",
            &src_dir,
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "src/ directory not found — are you in an Ironic project?",
            ),
        ));
    }

    writeln!(output, "ironic dev — watching for changes in src/").map_err(io_err)?;
    writeln!(output, "Press Ctrl+C to stop").map_err(io_err)?;

    let running = Arc::new(AtomicBool::new(true));
    let child = Arc::new(Mutex::new(None::<Child>));

    start_process(&root, &cargo_args.cargo_args, &child, output)?;

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
    let args_clone = cargo_args.cargo_args.clone();

    let restart = move || {
        kill_child(&child_clone);
        std::thread::sleep(Duration::from_millis(300));
        if running_clone.load(Ordering::SeqCst) {
            let _ = start_process(
                &root_clone,
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

    let cargo_toml = root.join("Cargo.toml");
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
    cargo_args: &[String],
    child: &Arc<Mutex<Option<Child>>>,
    output: &mut impl Write,
) -> Result<(), CliError> {
    writeln!(output, "\n🔨 Building...").map_err(io_err)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
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
