use super::super::common::naming;

const IRONIC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the content for a gRPC service's `Cargo.toml`.
pub(crate) fn app_manifest_grpc(names: &naming::Names) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = {{ workspace = true, features = ["grpc"] }}
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
tonic = {{ version = "0.14", features = ["transport", "codegen", "router"] }}
prost = "0.14"
tonic-prost = "0.14"
serde_json = "1"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"

[build-dependencies]
tonic-prost-build = "0.14"
"#,
        name = names.raw
    )
}

/// Returns the content for a gRPC service's `build.rs`.
pub(crate) fn app_build() -> String {
    r#"fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .compile_protos(&["proto/hello.proto"], &["proto"])?;
    Ok(())
}
"#
    .to_string()
}

/// Returns the content for a gRPC service's protobuf definition.
pub(crate) fn app_proto(_names: &naming::Names) -> String {
    r#"syntax = "proto3";

package hello;

service Greeter {
    rpc SayHello (HelloRequest) returns (HelloReply);
}

message HelloRequest {
    string name = 1;
}

message HelloReply {
    string message = 1;
}
"#
    .to_string()
}

/// Returns the content for a gRPC service's `src/main.rs`.
pub(crate) fn app_main_grpc(names: &naming::Names, port: u16) -> String {
    let version = IRONIC_VERSION;
    format!(
        r#"mod app;
mod app_service;
mod modules;
mod platform;

use std::sync::Arc;

use ironic::prelude::*;
use tonic::transport::Server;
use crate::app_service::AppService;

use app::AppModule;
use modules::greet::GreeterService;

pub mod hello {{
    tonic::include_proto!("hello");
}}

fn build_container() -> ironic::Container {{
    let graph = ironic::compile_module_graph(AppModule::definition())
        .expect("module graph must compile");
    let mut builder = ironic::ContainerBuilder::new();
    for module in graph.modules() {{
        for provider in module.providers() {{
            builder.register(provider.clone()).expect("provider registration");
        }}
    }}
    builder.build()
}}

#[ironic::main]
async fn main() {{
    dotenvy::dotenv().ok();
    platform::logging::init();

    let container = build_container();
    let greeter = container
        .resolve::<GreeterService>()
        .await
        .expect("GreeterService must be registered in AppModule");

    let addr: std::net::SocketAddr = platform::config::listen_addr("{port}")
        .parse()
        .expect("invalid address");

    println!("🚀 {name} → http://{{addr}} (ironic v{version})");

    Server::builder()
        .add_service(hello::greeter_server::GreeterServer::new(
            Arc::into_inner(greeter).expect("unique ownership"),
        ))
        .serve(addr)
        .await
        .expect("server failed");
}}
"#,
        name = names.raw,
        port = port
    )
}

/// Returns the content for a gRPC app's `src/app.rs` with DI registration.
pub(crate) fn app_module_grpc() -> String {
    r"use ironic::prelude::*;
use crate::app_service::AppService;
use crate::modules::greet::{GreeterService, GreetRepository};

#[derive(Module)]
#[module(
    providers = [AppService, GreeterService, GreetRepository],
    exports = [GreeterService],
)]
pub struct AppModule;
"
    .to_string()
}

/// Returns the content for `src/modules/mod.rs`.
pub(crate) fn app_modules_mod() -> String {
    "pub mod greet;\n".to_string()
}

/// Returns the content for `src/modules/greet/mod.rs`.
pub(crate) fn app_greet_mod() -> &'static str {
    "pub mod greeter_service;\npub mod greet_repository;\npub use greeter_service::GreeterService;\npub use greet_repository::GreetRepository;\n"
}

/// Returns the content for `src/greet/greeter_service.rs`.
pub(crate) fn app_greeter_service(_names: &naming::Names) -> String {
    r"use std::sync::Arc;

use ironic::prelude::*;
use tonic::{Request, Response, Status, async_trait};
use crate::hello;
use super::greet_repository::GreetRepository;

#[derive(Injectable)]
pub struct GreeterService {
    repo: Arc<GreetRepository>,
}

#[async_trait]
impl hello::greeter_server::Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<hello::HelloRequest>,
    ) -> Result<Response<hello::HelloReply>, Status> {
        let name = request.get_ref().name.clone();
        let message = self.repo.greet(&name);
        Ok(Response::new(hello::HelloReply { message }))
    }
}
"
    .to_string()
}

/// Returns the content for `src/greet/greet_repository.rs`.
pub(crate) fn app_greet_repository() -> String {
    r#"use ironic::prelude::*;

#[derive(Injectable)]
pub struct GreetRepository;

impl GreetRepository {
    pub fn greet(&self, name: &str) -> String {
        format!("Hello {name}!")
    }
}
"#
    .to_string()
}
