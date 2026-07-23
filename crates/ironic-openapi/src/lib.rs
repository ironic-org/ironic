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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_config_defaults() {
        let config = OpenApiConfig::new("My API", "1.0.0");
        assert_eq!(config.title(), "My API");
        assert_eq!(config.json_path_value(), "/openapi.json");
    }

    #[test]
    fn openapi_config_custom_path() {
        let config = OpenApiConfig::new("API", "2.0")
            .json_path("/docs/openapi.json")
            .description("An API description");
        assert_eq!(config.json_path_value(), "/docs/openapi.json");
    }

    #[test]
    fn openapi_error_display_and_clone() {
        let err = OpenApiError::InvalidPath { path: "relative".into() };
        let msg = err.to_string();
        assert!(msg.contains("RF_OPENAPI_INVALID_PATH"));

        let conflict = OpenApiError::EndpointConflict { path: "/api/items".into() };
        let msg2 = conflict.to_string();
        assert!(msg2.contains("RF_OPENAPI_ENDPOINT_CONFLICT"));

        let dup = OpenApiError::DuplicateOperationId { operation_id: "getItems".into() };
        let msg3 = dup.to_string();
        assert!(msg3.contains("RF_OPENAPI_DUPLICATE_OPERATION_ID"));

        // Verify Clone + Eq
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn openapi_config_via_builder() {
        let config = OpenApiConfig::new("API", "1.0.0").description("desc");
        let config = config.schema::<String>("MyString");
        let _ = config;
    }

    #[test]
    fn openapi_response_construction() {
        let _resp = OpenApiResponse::new("Created")
            .json::<String>()
            .example(serde_json::json!("\"ok\""));
    }

    #[test]
    fn openapi_request_body_construction() {
        let _body = OpenApiRequestBody::json::<i32>()
            .optional()
            .example(serde_json::json!(42));
    }

    #[test]
    fn openapi_operation_builder() {
        let op = OpenApiOperation::new()
            .summary("List items")
            .description("Returns a paginated list.")
            .operation_id("listItems")
            .tag("items")
            .deprecated();
        let _ = op;
    }

    #[test]
    fn openapi_parameter_builder() {
        let param = OpenApiParameter::new::<String>("id", ParameterLocation::Path)
            .description("Item ID");
        let _ = param;
    }
}
