//! Binary entry point for the `RustFrame` CLI.

fn main() {
    if let Err(error) = rustframe_cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
