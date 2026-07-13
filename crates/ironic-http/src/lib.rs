#![doc = "Transport-neutral HTTP contracts for Ironic."]

mod error;
mod extract;
mod handler;
mod observability;
mod pipeline;
mod request;
mod response;
mod route;

pub use error::HttpError;
pub use extract::{
    ExtractFuture, ExtractedValue, HeaderParameter, JsonBody, ParameterExtractor, PathParameter,
    QueryParameters,
};
pub use handler::{ErasedHandler, HandlerArguments, HandlerFuture, handler_fn};
pub use observability::{RequestId, RequestTracing};
pub use pipeline::{
    Guard, GuardDecision, GuardFuture, Interceptor, InterceptorNext, Middleware, MiddlewareNext,
    ParameterPipe, PipeFuture, PipelineComponents, PipelineFuture, pipe_fn,
};
pub use request::{FrameworkRequest, RequestContext};
pub use response::{FrameworkBody, FrameworkResponse, IntoFrameworkResponse, Json};
pub use route::{
    CompiledHttpApplication, CompiledRoute, ControllerDefinition, RouteDefinition, RouteError,
    RouteMetadata, compile_controller_routes,
};

/// The HTTP method used in route metadata.
pub use http::Method as HttpMethod;
/// The HTTP status code used by framework responses.
pub use http::StatusCode as HttpStatus;
/// The parsed request URI.
pub use http::Uri;
/// HTTP headers used by transport-neutral requests and responses.
pub use http::{HeaderMap, HeaderName, HeaderValue};
