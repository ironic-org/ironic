//! Binary entry point for the Ironic CLI.

fn main() {
    if let Err(error) = ironic::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
