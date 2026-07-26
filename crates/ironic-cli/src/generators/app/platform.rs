/// Returns the content for `src/platform/mod.rs`.
pub(crate) fn app_platform_mod() -> String {
    "pub mod config;\npub mod logging;\n".to_string()
}

/// Returns the content for `src/platform/logging.rs`.
pub(crate) fn app_platform_logging() -> String {
    r#"use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
"#
    .to_string()
}

/// Returns the content for `src/platform/config.rs`.
pub(crate) fn app_platform_config() -> String {
    r#"use std::env;

pub fn listen_addr(default_port: &str) -> String {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| default_port.into());
    format!("{host}:{port}")
}
"#
    .to_string()
}
