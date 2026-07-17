pub fn server_address() -> String {
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3002".into());
    format!("{}:{}", host, port)
}
