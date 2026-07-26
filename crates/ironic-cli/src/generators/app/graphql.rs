use super::super::common::naming;

/// Returns the content for a GraphQL service's `Cargo.toml`.
pub(crate) fn app_manifest_graphql(names: &naming::Names) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = {{ workspace = true, features = ["graphql"] }}
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"
"#,
        name = names.raw
    )
}

/// Returns the content for a GraphQL service's `src/main.rs`.
pub(crate) fn app_main_graphql(names: &naming::Names, port: u16) -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"mod app;
mod app_service;
mod platform;

use std::sync::Arc;

use ironic::prelude::*;
use async_graphql::{{EmptySubscription, Schema}};

use app::AppModule;
use app_service::AppService;

#[ironic::main]
async fn main() {{
    dotenvy::dotenv().ok();
    platform::logging::init();

    let schema = Schema::build(AppService::new(), AppService::new(), EmptySubscription)
        .data(AppModule)
        .finish();

    let addr = platform::config::listen_addr("{port}");
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(AxumAdapter::new().graphql_endpoint("/graphql", schema))
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 {name} → http://{{addr}}/graphql (ironic v{version})");
    app.listen(&addr).await.expect("server failed");
}}
"#,
        name = names.raw,
        port = port
    )
}

/// Returns the content for a GraphQL app's `src/app.rs`.
pub(crate) fn app_module_graphql() -> String {
    r"use ironic::prelude::*;
use crate::app_service::AppService;

#[derive(Module)]
#[module(
    providers = [AppService],
)]
pub struct AppModule;
"
    .to_string()
}

/// Returns the content for `src/app_service.rs` — query/mutation root.
pub(crate) fn app_service_graphql() -> String {
    r#"use async_graphql::{Object, Context, Result};

#[derive(Default)]
pub struct AppService;

#[Object]
impl AppService {
    async fn hello(&self, _ctx: &Context<'_>) -> Result<String> {
        Ok("Hello from GraphQL!".into())
    }
}

impl AppService {
    pub fn new() -> Self {
        Self
    }
}
"#
    .to_string()
}
