#![doc = "Ironic is a batteries-included, type-safe application framework for Rust."]

// Keep the source modules independent internally while presenting one public crate.
// These aliases also keep generated and hand-written framework internals on stable paths.
extern crate self as rustframe_core;
extern crate self as rustframe_di;
extern crate self as rustframe_http;
extern crate self as rustframe_platform;
extern crate self as rustframe_platform_axum;

#[cfg(feature = "auth")]
#[path = "../crates/ironic-auth/src/lib.rs"]
pub mod auth;
#[path = "../crates/ironic-cli/src/lib.rs"]
mod cli_impl;
#[path = "../crates/ironic-common/src/lib.rs"]
mod common;
#[path = "../crates/ironic-config/src/lib.rs"]
mod config_impl;
#[path = "../crates/ironic-core/src/lib.rs"]
mod core;
#[path = "../crates/ironic-di/src/lib.rs"]
mod di;
#[cfg(any(
    feature = "queues",
    feature = "microservices",
    feature = "cqrs",
    feature = "sagas",
    feature = "grpc",
    feature = "graphql"
))]
#[path = "../crates/ironic-distributed/src/lib.rs"]
pub mod distributed;
#[cfg(any(feature = "plugins", feature = "devtools"))]
#[path = "../crates/ironic-devtools/src/lib.rs"]
pub mod ecosystem;
#[path = "../crates/ironic-http/src/lib.rs"]
mod http_impl;
#[path = "../crates/ironic-integrations/src/lib.rs"]
pub mod integrations;
#[path = "../crates/ironic-openapi/src/lib.rs"]
mod openapi;
#[path = "../crates/ironic-platform/src/lib.rs"]
mod platform;
#[path = "../crates/ironic-platform-axum/src/lib.rs"]
mod platform_axum;
#[cfg(any(
    feature = "cache",
    feature = "scheduling",
    feature = "events",
    feature = "realtime"
))]
#[path = "../crates/ironic-services/src/lib.rs"]
pub mod services;
#[path = "../crates/ironic-testing/src/lib.rs"]
mod testing;

pub use cli_impl::{CliError, cli, generators, run, run_with};
pub use common::*;
pub use config_impl::*;
pub use core::*;
pub use di::*;
pub use http_impl::*;
pub use ironic_macros::{
    Injectable, Module, OpenApiSchema, body, controller, delete, get, head, header, main, options,
    param, patch, post, put, query, routes, use_guard, use_interceptor,
};
pub use openapi::*;
pub use platform::*;
pub use platform_axum::*;
pub use testing::*;

/// Implementation details used by generated code.
#[doc(hidden)]
pub mod __private {
    use std::future::Future;

    pub use serde_json;

    /// Runs the future produced by a macro-generated application entry point.
    pub fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Ironic failed to initialize its Tokio runtime")
            .block_on(future)
    }
}

/// Commonly used Ironic types and macros.
pub mod prelude {
    pub use crate::{
        ConfigurationError, ConfigurationLoader, ControllerDefinition, Dependency,
        FrameworkApplication, FrameworkError, FrameworkResult, Guard, GuardDecision, GuardFuture,
        HeaderParameter, HealthModule, HealthStatus, HttpError, HttpMethod, HttpPlatformAdapter,
        HttpPlatformApplication, Injectable, Interceptor, InterceptorNext, Json, JsonBody,
        LifecycleDefinition, Middleware, Module, ModuleDefinition, OnApplicationBootstrap,
        OnApplicationShutdown, OnModuleDestroy, OnModuleInit, OpenApiSchema, ParameterPipe,
        PathParameter, PipelineFuture, ProviderDefinition, QueryParameters, RequestContext,
        RequestId, RequestScope, RequestTracing, RouteDefinition, RouteMetadata, Scope, Secret,
        SecretString, ShutdownSignal, ValidateConfiguration, body, controller, delete, get,
        handler_fn, head, header, options, param, patch, pipe_fn, post, put, query, routes,
        use_guard, use_interceptor,
    };
}
