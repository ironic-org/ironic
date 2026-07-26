use crate::generators::common::naming;

const IRONIC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the content for the app's entry point `src/main.rs`.
pub(crate) fn app_main(names: &naming::Names, port: u16) -> String {
    let version = IRONIC_VERSION;
    format!(
        r#"mod app;
mod app_controller;
mod app_service;
mod platform;

use ironic::prelude::*;

use app::AppModule;

#[ironic::main]
async fn main() {{
    ironic::dotenvy::dotenv().ok();
    platform::logging::init();

    let addr = platform::config::listen_addr("{port}");
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{addr}} (ironic v{version})");
    app.listen(&addr).await.expect("server failed");
}}
"#,
        name = names.raw,
        port = port
    )
}

/// Returns the content for the root module `src/app.rs`.
pub(crate) fn app_module() -> &'static str {
    "use ironic::prelude::*;
use crate::app_controller::AppController;
use crate::app_service::AppService;

#[derive(Module)]
#[module(
    controllers = [AppController],
    providers = [AppService],
)]
pub struct AppModule;
"
}

/// Returns the content for `src/app_controller.rs` — root controller (NestJS-style).
pub(crate) fn app_controller() -> &'static str {
    r#"use std::sync::Arc;
use ironic::prelude::*;
use crate::app_service::AppService;

#[controller("/")]
#[derive(Injectable)]
pub struct AppController {
    service: Arc<AppService>,
}

#[routes]
impl AppController {
    #[get]
    async fn index(&self) -> Result<Json<ironic::Value>, HttpError> {
        let greeting = self.service.greeting();
        Ok(Json(greeting))
    }
}
"#
}

/// Returns the content for `src/app_service.rs` — root service (NestJS-style).
pub(crate) fn app_service(name: &str, version: &str) -> String {
    format!(
        r#"use ironic::prelude::*;

#[derive(Injectable)]
pub struct AppService;

impl AppService {{
    pub fn greeting(&self) -> ironic::Value {{
        ironic::json::json!({{
            "app": "{name}",
            "framework": "Ironic",
            "version": "{version}",
            "status": "running"
        }})
    }}
}}
"#,
    )
}

/// Returns the content for the app's `Cargo.toml`.
pub(crate) fn app_manifest(names: &naming::Names) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = {{ workspace = true }}
"#,
        name = names.raw
    )
}

/// Returns the content for the app's multi-stage `Dockerfile`.
pub(crate) fn app_dockerfile(names: &naming::Names, port: u16) -> String {
    let bin = &names.raw;
    format!(
        r#"FROM rust:1.97-alpine AS builder
WORKDIR /app

# musl + static deps for fully linked binaries
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && \
    rustup target add x86_64-unknown-linux-musl

# Cache dependencies first (dummy source, then real build)
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {{}}" > src/main.rs
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; true

# Real build with actual source
COPY src ./src
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/{bin} /{bin}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT={port}
EXPOSE {port}
CMD ["/{bin}"]
"#,
    )
}

/// Returns the content for the app's `.env` file with default dev values.
pub(crate) fn app_env(names: &naming::Names, port: u16) -> String {
    format!(
        r"SERVER_HOST=0.0.0.0
SERVER_PORT={port}
RUST_LOG=info
CORS_ORIGINS=[]
RATE_LIMIT_MAX=100
DATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/{name}
",
        port = port,
        name = names.raw
    )
}
