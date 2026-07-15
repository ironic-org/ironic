pub fn env(key: &str) -> Option<String> {
    dotenvy::var(key).ok()
}

pub fn env_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_json_array(key: &str) -> Vec<String> {
    env(key)
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn server_address() -> String {
    let host = env("SERVER_HOST").unwrap_or_else(|| "127.0.0.1".into());
    let port = env("SERVER_PORT").unwrap_or_else(|| "3000".into());
    format!("{}:{}", host, port)
}
