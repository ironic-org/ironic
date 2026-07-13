#![doc = "The public facade for the RustFrame application framework."]

pub use rustframe_common::{FrameworkError, FrameworkResult};
pub use rustframe_config::{
    ConfigurationError, ConfigurationLoader, Secret, SecretString, ValidateConfiguration,
};
pub use rustframe_core::{
    ApplicationError, FrameworkApplication, FrameworkApplicationBuilder, HealthModule,
    HealthStatus, LifecycleDefinition, LifecycleError, Module, ModuleDefinition,
    OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
};
pub use rustframe_di::{
    Container, Dependency, ProviderDefinition, ProviderKey, ResolveError, Scope,
};
pub use rustframe_http::{
    ControllerDefinition, FrameworkRequest, FrameworkResponse, Guard, GuardDecision, GuardFuture,
    HeaderParameter, HttpError, HttpMethod, HttpStatus, Interceptor, InterceptorNext, Json,
    JsonBody, Middleware, PathParameter, PipelineFuture, QueryParameters, RequestContext,
    RequestId, RequestTracing, RouteDefinition, RouteMetadata, handler_fn, pipe_fn,
};
pub use rustframe_macros::{
    Injectable, Module, body, controller, delete, get, head, header, main, options, param, patch,
    post, put, query, routes, use_guard, use_interceptor,
};
pub use rustframe_platform::{HttpPlatformAdapter, HttpPlatformApplication, ShutdownSignal};

/// Implementation details used by generated code.
#[doc(hidden)]
pub mod __private {
    use std::future::Future;

    /// Runs the future produced by a macro-generated application entry point.
    pub fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("RustFrame failed to initialize its Tokio runtime")
            .block_on(future)
    }
}

/// Commonly used `RustFrame` types.
pub mod prelude {
    pub use rustframe_common::{FrameworkError, FrameworkResult};
    pub use rustframe_config::{
        ConfigurationError, ConfigurationLoader, Secret, SecretString, ValidateConfiguration,
    };
    pub use rustframe_core::{
        FrameworkApplication, HealthModule, HealthStatus, LifecycleDefinition, Module,
        ModuleDefinition, OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy,
        OnModuleInit,
    };
    pub use rustframe_di::{Dependency, ProviderDefinition, Scope};
    pub use rustframe_http::{
        ControllerDefinition, Guard, GuardDecision, GuardFuture, HeaderParameter, HttpError,
        HttpMethod, Interceptor, InterceptorNext, Json, JsonBody, Middleware, PathParameter,
        PipelineFuture, QueryParameters, RequestContext, RequestId, RequestTracing,
        RouteDefinition, RouteMetadata, handler_fn, pipe_fn,
    };
    pub use rustframe_macros::{
        Injectable, Module, body, controller, delete, get, head, header, options, param, patch,
        post, put, query, routes, use_guard, use_interceptor,
    };
    pub use rustframe_platform::{HttpPlatformAdapter, HttpPlatformApplication, ShutdownSignal};
}
