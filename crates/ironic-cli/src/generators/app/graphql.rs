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
async-graphql = {{ workspace = true }}
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

use app::AppModule;
use app_service::AppService;

type GqlSchema = ironic::async_graphql::Schema<AppService, ironic::async_graphql::EmptyMutation, ironic::async_graphql::EmptySubscription>;

#[ironic::main]
async fn main() {{
    ironic::dotenvy::dotenv().ok();
    platform::logging::init();

    let schema = Arc::new(
        ironic::async_graphql::Schema::build(AppService, ironic::async_graphql::EmptyMutation, ironic::async_graphql::EmptySubscription)
            .data(AppModule)
            .finish()
    );

    async fn graphql_handler(
        schema: ironic::axum::Extension<Arc<GqlSchema>>,
        request: ironic::axum::Json<ironic::async_graphql::Request>,
    ) -> ironic::axum::Json<ironic::async_graphql::Response> {{
        let response = schema.execute(request.0).await;
        ironic::axum::Json(response)
    }}

    let addr = platform::config::listen_addr("{port}");
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(
            AxumAdapter::new().configure_router(move |router| {{
                router
                    .route("/graphql", ironic::axum::routing::post(graphql_handler))
                    .layer(ironic::axum::Extension(schema))
            }})
        )
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
    r#"use ironic::prelude::*;
use async_graphql::{Object, Context, Result};

#[derive(Injectable)]
pub struct AppService;

#[Object]
impl AppService {
    async fn hello(&self, _ctx: &Context<'_>) -> Result<String> {
        Ok("Hello from GraphQL!".into())
    }
}
"#
    .to_string()
}
