#![doc = "`OpenAPI` generation and Swagger UI integration for Ironic."]

mod axum;
mod document;
mod schema;

pub use axum::{OpenApiAxumAdapter, OpenApiAxumApplication, OpenApiAxumError, OpenApiAxumExt};
pub use document::{
    OpenApiConfig, OpenApiDocument, OpenApiError, OpenApiOperation, OpenApiParameter,
    OpenApiRequestBody, OpenApiResponse, OpenApiRouteExt, ParameterLocation, SecurityScheme,
};
pub use schema::OpenApiSchema;
