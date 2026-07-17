#![doc = "Ironic is a batteries-included, type-safe application framework for Rust."]

// Keep the source modules independent internally while presenting one public crate.
// These aliases also keep generated and hand-written framework internals on stable paths.
#[allow(unused_extern_crates)]
extern crate self as ironic_core;
#[allow(unused_extern_crates)]
extern crate self as ironic_di;
#[allow(unused_extern_crates)]
extern crate self as ironic_http;
#[allow(unused_extern_crates)]
extern crate self as ironic_platform;
#[allow(unused_extern_crates)]
extern crate self as ironic_platform_axum;

#[cfg(feature = "auth")]
#[path = "../crates/ironic-auth/src/lib.rs"]
pub mod auth;
#[cfg(all(
    feature = "cache",
    any(feature = "redis", feature = "application-services")
))]
mod cache_interceptor;
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
#[cfg(feature = "metrics")]
#[path = "../crates/ironic-metrics/src/lib.rs"]
pub mod metrics;
#[cfg(feature = "resilience")]
#[path = "../crates/ironic-resilience/src/lib.rs"]
pub mod resilience;
#[cfg(feature = "telemetry")]
#[path = "../crates/ironic-telemetry/src/lib.rs"]
pub mod telemetry;

#[cfg(any(feature = "plugins", feature = "devtools"))]
#[path = "../crates/ironic-devtools/src/lib.rs"]
pub mod ecosystem;
#[path = "../crates/ironic-http/src/lib.rs"]
mod http_impl;
#[path = "../crates/ironic-integrations/src/lib.rs"]
pub mod integrations;
#[cfg(feature = "logging")]
#[path = "../crates/ironic-logging/src/lib.rs"]
pub mod logging;
#[cfg(feature = "openapi")]
#[path = "../crates/ironic-openapi/src/lib.rs"]
mod openapi;
#[path = "../crates/ironic-platform/src/lib.rs"]
mod platform;
#[path = "../crates/ironic-platform-axum/src/lib.rs"]
mod platform_axum;
#[cfg(feature = "security")]
#[path = "../crates/ironic-security/src/lib.rs"]
pub mod security;
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
pub use http_impl::{
    CacheMetadata, ExceptionFilter, FilterContext, VersionMetadata, VersioningStrategy,
};
pub use ironic_macros::{
    Injectable, Module, OpenApiSchema, Serializable, api, body, cache, controller, cron, custom,
    delete, get, head, header, interval, main, options, param, patch, pipe, post, put, query,
    req_body, resp, routes, subscribe_message, r#test, timeout, use_guard, use_interceptor,
    web_socket_gateway,
};
#[cfg(feature = "openapi")]
pub use openapi::*;
pub use platform::*;
pub use platform_axum::*;
pub use testing::*;
#[cfg(feature = "multipart")]
pub use {
    http_impl::MultipartConfig, http_impl::MultipartForm, http_impl::MultipartFormData,
    http_impl::UploadedFile,
};

#[cfg(all(
    feature = "cache",
    any(feature = "redis", feature = "application-services")
))]
pub use cache_interceptor::CacheInterceptor;

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

/// Creates a custom parameter decorator that can be used with `#[custom(DecoratorName)]`
/// in route handler signatures.
///
/// The macro defines a type alias so that the decorator name can be used as the argument
/// to `#[custom(...)]`. The extractor type must implement [`ParameterExtractor`] and
/// provide a `::new()` constructor.
///
/// # Example
///
/// ```rust
/// use ironic::{ParameterExtractor, RequestContext, ExtractFuture, create_param_decorator};
/// use std::sync::Arc;
///
/// struct CurrentUser;
///
/// impl ParameterExtractor for CurrentUser {
///     fn extract<'a>(&'a self, _context: &'a mut RequestContext) -> ExtractFuture<'a> {
///         Box::pin(async move { Ok(Box::new("user-123".to_string()) as Box<dyn std::any::Any + Send>) })
///     }
///     fn description(&self) -> &'static str { "current_user" }
/// }
///
/// create_param_decorator!(current_user, CurrentUser);
/// ```
#[macro_export]
macro_rules! create_param_decorator {
    ($name:ident, $extractor:ty) => {
        #[doc = concat!("Custom parameter decorator type for `", stringify!($name), "`.")]
        pub type $name = $extractor;
    };
}

/// Commonly used Ironic types and macros.
pub mod prelude {
    #[cfg(feature = "hot-reload")]
    pub use crate::ConfigWatcher;
    #[cfg(feature = "openapi")]
    pub use crate::OpenApiSchema;
    #[cfg(feature = "validation")]
    pub use crate::ValidationPipe;
    #[cfg(all(
        feature = "cache",
        any(feature = "redis", feature = "application-services")
    ))]
    pub use crate::cache_interceptor::CacheInterceptor;
    #[cfg(feature = "logging")]
    pub use crate::logging::{
        LogEntry, LogStorage, StorageError, TimeSeriesConfig, TimeSeriesModule,
    };
    pub use crate::{
        AxumAdapter, BuildInfo, CacheMetadata, CompiledHttpApplication, ConfigurationError,
        ConfigurationLoader, ControllerDefinition, Dependency, ExceptionFilter, FeatureToggle,
        FilterContext, FrameworkApplication, FrameworkError, FrameworkResult, Guard, GuardDecision,
        GuardFuture, HeaderParameter, HealthModule, HealthStatus, HttpError, HttpMethod,
        HttpPlatformAdapter, HttpPlatformApplication, Injectable, Interceptor, InterceptorNext,
        Json, JsonBody, LifecycleDefinition, Middleware, Module, ModuleDefinition, ModuleRef,
        OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
        ParameterPipe, PathParameter, PipelineFuture, ProviderDefinition, QueryParameters,
        RequestContext, RequestId, RequestLogging, RequestScope, RequestTracing, RouteDefinition, RouteMetadata,
        Scope, Secret, SecretString, Serializable, ShutdownSignal, ValidateConfiguration,
        VersionMetadata, VersioningStrategy, WsGatewayDefinition, api, body, cache, controller,
        create_param_decorator, cron, custom, delete, get, handler_fn, head, header, interval,
        options, param, patch, pipe, pipe_fn, post, put, query, req_body, resp, routes,
        subscribe_message, timeout, use_guard, use_interceptor, web_socket_gateway,
    };
    #[cfg(feature = "serialization")]
    pub use crate::{FieldRule, FieldRules, SerializeInterceptor, set_current_roles};
    #[cfg(feature = "multipart")]
    pub use crate::{MultipartConfig, MultipartForm, MultipartFormData, UploadedFile};
}
