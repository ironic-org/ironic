#![doc = "Transport-neutral HTTP contracts for Ironic."]

mod error;
mod exception_filter;
mod extract;
mod extractors;
mod handler;
mod metadata;
#[cfg(feature = "multipart")]
mod multipart;
mod observability;
mod pipeline;
mod pipes;
mod request;
mod response;
mod route;
#[cfg(feature = "serialization")]
mod serialization;
#[cfg(feature = "sqlx")]
mod sqlx;

pub use error::HttpError;
pub(crate) use exception_filter::ExceptionFilterSet;
pub use exception_filter::{ExceptionExt, ExceptionFilter, FilterContext};
pub use extract::{
    ExtractFuture, ExtractedValue, FormBody, HeaderParameter, JsonBody, ParameterExtractor,
    PathParameter, QueryParameters,
};
pub use extractors::Pagination;
pub use handler::{ErasedHandler, HandlerArguments, HandlerFuture, handler_fn};
pub use metadata::{CacheMetadata, VersionMetadata, VersioningStrategy};
#[cfg(feature = "multipart")]
pub use multipart::{MultipartConfig, MultipartForm, MultipartFormData, UploadedFile};
pub use observability::{RequestId, RequestLogging, RequestTracing};
pub use pipeline::{
    Guard, GuardDecision, GuardFuture, Interceptor, InterceptorNext, Middleware, MiddlewareNext,
    ParameterPipe, PipeFuture, PipelineComponents, PipelineFuture, pipe_fn,
};
pub use pipes::{ParseBoolPipe, ParseFloatPipe, ParseIntPipe, parse_bool, parse_float, parse_int};
#[cfg(feature = "uuid")]
pub use pipes::{ParseUUIDPipe, parse_uuid};
#[cfg(feature = "validation")]
pub use pipes::{ValidationPipe, validate};
pub use request::{Request, RequestContext};
pub use response::{Body, IntoResponse, Json, Response};
pub use route::{
    CompiledHttpApplication, CompiledRoute, ControllerDefinition, RouteDefinition, RouteError,
    RouteMetadata, WsGatewayDefinition, compile_controller_routes,
};
#[cfg(feature = "serialization")]
pub use serialization::{FieldRule, FieldRules, SerializeInterceptor, set_current_roles};
#[cfg(feature = "sqlx")]
pub use sqlx::{SqlxErrorExt, SqlxResultExt};

/// The HTTP method used in route metadata.
pub use http::Method as HttpMethod;
/// The HTTP status code used by framework responses.
pub use http::StatusCode as HttpStatus;
/// The parsed request URI.
pub use http::Uri;
/// HTTP headers used by transport-neutral requests and responses.
pub use http::{HeaderMap, HeaderName, HeaderValue};
