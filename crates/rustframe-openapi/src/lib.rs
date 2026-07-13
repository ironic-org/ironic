#![doc = "`OpenAPI` generation and Swagger UI integration for `RustFrame`."]

mod axum;
mod document;
mod schema;

pub use axum::{OpenApiAxumAdapter, OpenApiAxumApplication, OpenApiAxumError, OpenApiAxumExt};
pub use document::{
    OpenApiConfig, OpenApiDocument, OpenApiError, OpenApiOperation, OpenApiParameter,
    OpenApiRequestBody, OpenApiResponse, OpenApiRouteExt, ParameterLocation, SecurityScheme,
};
pub use rustframe_openapi_macros::OpenApiSchema;
pub use schema::OpenApiSchema;

/// Dependencies used by generated schema implementations.
#[doc(hidden)]
pub mod __private {
    pub use serde_json;
}
