#![doc = "Transport-neutral HTTP contracts for Ironic."]

mod error;
mod exception_filter;
mod extract;
mod handler;
mod metadata;
mod observability;
mod pipeline;
mod pipes;
mod request;
mod response;
mod route;
#[cfg(feature = "serialization")]
mod serialization;

pub use error::HttpError;
pub use exception_filter::{ExceptionFilter, FilterContext};
pub(crate) use exception_filter::ExceptionFilterSet;
pub use metadata::{CacheMetadata, VersionMetadata, VersioningStrategy};
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
pub use pipes::{
    ParseBoolPipe, ParseFloatPipe, ParseIntPipe, parse_bool, parse_float, parse_int,
};
#[cfg(feature = "uuid")]
pub use pipes::{ParseUUIDPipe, parse_uuid};
#[cfg(feature = "validation")]
pub use pipes::{ValidationPipe, validate};
pub use request::{FrameworkRequest, RequestContext};
pub use response::{FrameworkBody, FrameworkResponse, IntoFrameworkResponse, Json};
pub use route::{
    CompiledHttpApplication, CompiledRoute, ControllerDefinition, RouteDefinition, RouteError,
    RouteMetadata, compile_controller_routes,
};
#[cfg(feature = "serialization")]
pub use serialization::{FieldRule, FieldRules, SerializeInterceptor, set_current_roles};

/// The HTTP method used in route metadata.
pub use http::Method as HttpMethod;
/// The HTTP status code used by framework responses.
pub use http::StatusCode as HttpStatus;
/// The parsed request URI.
pub use http::Uri;
/// HTTP headers used by transport-neutral requests and responses.
pub use http::{HeaderMap, HeaderName, HeaderValue};
