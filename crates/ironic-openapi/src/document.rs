use std::collections::{BTreeMap, HashSet};

use ironic_http::{CompiledHttpApplication, HttpMethod, RouteDefinition};
use serde_json::{Map, Value, json};

use crate::OpenApiSchema;

/// An `OpenAPI` document generation failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum OpenApiError {
    /// A document or UI path is not absolute.
    #[error("RF_OPENAPI_INVALID_PATH: `{path}` must begin with `/`")]
    InvalidPath {
        /// The invalid path.
        path: String,
    },
    /// Two operations explicitly declare the same operation ID.
    #[error("RF_OPENAPI_DUPLICATE_OPERATION_ID: `{operation_id}` is declared more than once")]
    DuplicateOperationId {
        /// The duplicated operation ID.
        operation_id: String,
    },
    /// A generated endpoint overlaps an existing framework route.
    #[error(
        "RF_OPENAPI_ENDPOINT_CONFLICT: generated endpoint `{path}` conflicts with an application route"
    )]
    EndpointConflict {
        /// The conflicting path.
        path: String,
    },
}

/// Configuration for a generated `OpenAPI` 3.1 document.
#[derive(Clone, Debug)]
pub struct OpenApiConfig {
    title: String,
    version: String,
    description: Option<String>,
    json_path: String,
    schemas: BTreeMap<String, Value>,
    security_schemes: BTreeMap<String, SecurityScheme>,
}

impl OpenApiConfig {
    /// Creates document configuration with a title and API version.
    #[must_use]
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: None,
            json_path: "/openapi.json".to_owned(),
            schemas: BTreeMap::new(),
            security_schemes: BTreeMap::new(),
        }
    }

    /// Sets the API description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the endpoint that serves the generated JSON document.
    #[must_use]
    pub fn json_path(mut self, path: impl Into<String>) -> Self {
        self.json_path = path.into();
        self
    }

    /// Registers a reusable component schema.
    #[must_use]
    pub fn schema<T: OpenApiSchema>(mut self, name: impl Into<String>) -> Self {
        self.schemas.insert(name.into(), T::openapi_schema());
        self
    }

    /// Registers a named authentication scheme.
    #[must_use]
    pub fn security_scheme(mut self, name: impl Into<String>, scheme: SecurityScheme) -> Self {
        self.security_schemes.insert(name.into(), scheme);
        self
    }

    /// Returns the configured JSON endpoint.
    #[must_use]
    pub fn json_path_value(&self) -> &str {
        &self.json_path
    }

    /// Returns the document title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn validate(&self) -> Result<(), OpenApiError> {
        validate_absolute_path(&self.json_path)
    }
}

/// A reusable `OpenAPI` authentication scheme.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SecurityScheme {
    /// An API key transported in a header, query parameter, or cookie.
    ApiKey {
        /// Parameter name.
        name: String,
        /// `OpenAPI` location: `header`, `query`, or `cookie`.
        location: String,
    },
    /// HTTP bearer authentication.
    HttpBearer {
        /// Optional bearer token format, such as `JWT`.
        bearer_format: Option<String>,
    },
    /// OAuth 2 authorization-code flow.
    OAuth2AuthorizationCode {
        /// Authorization endpoint.
        authorization_url: String,
        /// Token endpoint.
        token_url: String,
        /// Scope name to description mapping.
        scopes: BTreeMap<String, String>,
    },
}

impl SecurityScheme {
    fn as_json(&self) -> Value {
        match self {
            Self::ApiKey { name, location } => {
                json!({ "type": "apiKey", "name": name, "in": location })
            }
            Self::HttpBearer { bearer_format } => json!({
                "type": "http",
                "scheme": "bearer",
                "bearerFormat": bearer_format
            }),
            Self::OAuth2AuthorizationCode {
                authorization_url,
                token_url,
                scopes,
            } => json!({
                "type": "oauth2",
                "flows": {
                    "authorizationCode": {
                        "authorizationUrl": authorization_url,
                        "tokenUrl": token_url,
                        "scopes": scopes
                    }
                }
            }),
        }
    }
}

/// Documentation for one response status.
#[derive(Clone, Debug)]
pub struct OpenApiResponse {
    description: String,
    schema: Option<Value>,
    example: Option<Value>,
}

impl OpenApiResponse {
    /// Creates a response description.
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            schema: None,
            example: None,
        }
    }

    /// Sets the JSON response schema.
    #[must_use]
    pub fn json<T: OpenApiSchema>(mut self) -> Self {
        self.schema = Some(T::openapi_schema());
        self
    }

    /// Sets a serialized example value.
    #[must_use]
    pub fn example(mut self, example: Value) -> Self {
        self.example = Some(example);
        self
    }

    fn as_json(&self) -> Value {
        let mut media = Map::new();
        if let Some(schema) = &self.schema {
            media.insert("schema".to_owned(), schema.clone());
        }
        if let Some(example) = &self.example {
            media.insert("example".to_owned(), example.clone());
        }
        let mut response = json!({ "description": self.description });
        if !media.is_empty() {
            response["content"] = json!({ "application/json": media });
        }
        response
    }
}

/// Documentation for a JSON request body.
#[derive(Clone, Debug)]
pub struct OpenApiRequestBody {
    required: bool,
    schema: Value,
    example: Option<Value>,
}

impl OpenApiRequestBody {
    /// Creates a required JSON request body for `T`.
    #[must_use]
    pub fn json<T: OpenApiSchema>() -> Self {
        Self {
            required: true,
            schema: T::openapi_schema(),
            example: None,
        }
    }

    /// Marks the request body as optional.
    #[must_use]
    pub const fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Sets a serialized example value.
    #[must_use]
    pub fn example(mut self, example: Value) -> Self {
        self.example = Some(example);
        self
    }

    fn as_json(&self) -> Value {
        let mut media = json!({ "schema": self.schema });
        if let Some(example) = &self.example {
            media["example"] = example.clone();
        }
        json!({
            "required": self.required,
            "content": { "application/json": media }
        })
    }
}

/// The location of a documented operation parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParameterLocation {
    /// A path-template parameter.
    Path,
    /// A query-string parameter.
    Query,
    /// A request header.
    Header,
}

impl ParameterLocation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
        }
    }
}

/// Documentation for one operation parameter.
#[derive(Clone, Debug)]
pub struct OpenApiParameter {
    name: String,
    location: ParameterLocation,
    required: bool,
    schema: Value,
    description: Option<String>,
}

impl OpenApiParameter {
    /// Creates a parameter with a schema inferred from `T`.
    #[must_use]
    pub fn new<T: OpenApiSchema>(name: impl Into<String>, location: ParameterLocation) -> Self {
        Self {
            name: name.into(),
            location,
            required: location == ParameterLocation::Path,
            schema: T::openapi_schema(),
            description: None,
        }
    }

    /// Marks a non-path parameter as required or optional.
    #[must_use]
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the parameter description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    fn as_json(&self) -> Value {
        json!({
            "name": self.name,
            "in": self.location.as_str(),
            "required": self.required,
            "description": self.description,
            "schema": self.schema
        })
    }
}

/// `OpenAPI` metadata attached to a compiled route.
#[derive(Clone, Debug, Default)]
pub struct OpenApiOperation {
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    tags: Vec<String>,
    parameters: Vec<OpenApiParameter>,
    request_body: Option<OpenApiRequestBody>,
    responses: BTreeMap<String, OpenApiResponse>,
    security: Vec<(String, Vec<String>)>,
    deprecated: bool,
}

impl OpenApiOperation {
    /// Creates empty operation metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the short operation summary.
    #[must_use]
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Sets the detailed operation description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets a stable operation identifier.
    #[must_use]
    pub fn operation_id(mut self, operation_id: impl Into<String>) -> Self {
        self.operation_id = Some(operation_id.into());
        self
    }

    /// Adds a grouping tag.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Adds a documented parameter.
    #[must_use]
    pub fn parameter(mut self, parameter: OpenApiParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Sets the JSON request body.
    #[must_use]
    pub fn request_body(mut self, request_body: OpenApiRequestBody) -> Self {
        self.request_body = Some(request_body);
        self
    }

    /// Adds a response for an HTTP status code or `default`.
    #[must_use]
    pub fn response(mut self, status: impl Into<String>, response: OpenApiResponse) -> Self {
        self.responses.insert(status.into(), response);
        self
    }

    /// Requires a named authentication scheme and optional scopes.
    #[must_use]
    pub fn security(
        mut self,
        scheme: impl Into<String>,
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.security
            .push((scheme.into(), scopes.into_iter().map(Into::into).collect()));
        self
    }

    /// Marks this operation as deprecated.
    #[must_use]
    pub const fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
}

/// Adds `OpenAPI`-specific metadata without coupling the HTTP kernel to `OpenAPI`.
pub trait OpenApiRouteExt {
    /// Attaches operation metadata to this route definition.
    #[must_use]
    fn openapi(self, operation: OpenApiOperation) -> Self;
}

impl OpenApiRouteExt for RouteDefinition {
    fn openapi(self, operation: OpenApiOperation) -> Self {
        self.metadata(operation)
    }
}

/// A generated, serializable `OpenAPI` document.
#[derive(Clone, Debug)]
pub struct OpenApiDocument {
    value: Value,
}

impl OpenApiDocument {
    /// Discovers compiled framework routes and generates an `OpenAPI` 3.1 document.
    ///
    /// Iterates all registered routes, collects their [`OpenApiOperation`] metadata,
    /// and builds a complete `OpenAPI` document with paths, components, and security
    /// definitions.
    ///
    /// # Errors
    ///
    /// Returns [`OpenApiError`] for:
    /// - Invalid paths (not starting with `/`)
    /// - Duplicate explicit operation IDs
    pub fn from_application(
        application: &CompiledHttpApplication,
        config: &OpenApiConfig,
    ) -> Result<Self, OpenApiError> {
        config.validate()?;
        let mut paths = Map::new();
        let mut operation_ids = HashSet::new();

        for (index, route) in application.routes().iter().enumerate() {
            let metadata = route
                .metadata()
                .get::<OpenApiOperation>()
                .cloned()
                .unwrap_or_default();
            let operation_id = metadata
                .operation_id
                .clone()
                .unwrap_or_else(|| default_operation_id(route.handler_name(), index));
            if !operation_ids.insert(operation_id.clone()) {
                return Err(OpenApiError::DuplicateOperationId { operation_id });
            }
            let path = openapi_path(route.path());
            let operation = operation_json(&metadata, &operation_id, &path);
            let path_item = paths
                .entry(path)
                .or_insert_with(|| Value::Object(Map::new()));
            if let Some(path_item) = path_item.as_object_mut() {
                path_item.insert(method_name(route.method()), operation);
            }
        }

        let schemas = config.schemas.clone();
        let security_schemes = config
            .security_schemes
            .iter()
            .map(|(name, scheme)| (name.clone(), scheme.as_json()))
            .collect::<Map<_, _>>();
        Ok(Self {
            value: json!({
                "openapi": "3.1.0",
                "info": {
                    "title": config.title,
                    "version": config.version,
                    "description": config.description,
                },
                "paths": paths,
                "components": {
                    "schemas": schemas,
                    "securitySchemes": security_schemes,
                }
            }),
        })
    }

    /// Returns the generated JSON value.
    #[must_use]
    pub const fn as_value(&self) -> &Value {
        &self.value
    }

    /// Consumes the document and returns its JSON value.
    #[must_use]
    pub fn into_value(self) -> Value {
        self.value
    }

    /// Serializes the document as pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the JSON serializer cannot encode the document.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.value)
    }
}

fn operation_json(metadata: &OpenApiOperation, operation_id: &str, path: &str) -> Value {
    let mut parameters = metadata
        .parameters
        .iter()
        .map(OpenApiParameter::as_json)
        .collect::<Vec<_>>();
    for name in path_parameter_names(path) {
        let documented = metadata.parameters.iter().any(|parameter| {
            parameter.location == ParameterLocation::Path && parameter.name == name
        });
        if !documented {
            parameters
                .push(OpenApiParameter::new::<String>(name, ParameterLocation::Path).as_json());
        }
    }

    let responses = if metadata.responses.is_empty() {
        BTreeMap::from([(
            "200".to_owned(),
            OpenApiResponse::new("Successful response"),
        )])
    } else {
        metadata.responses.clone()
    };
    let responses = responses
        .iter()
        .map(|(status, response)| (status.clone(), response.as_json()))
        .collect::<Map<_, _>>();
    let security = metadata
        .security
        .iter()
        .map(|(scheme, scopes)| {
            let mut requirement = Map::new();
            requirement.insert(scheme.clone(), json!(scopes));
            Value::Object(requirement)
        })
        .collect::<Vec<_>>();

    let mut operation = json!({
        "operationId": operation_id,
        "summary": metadata.summary,
        "description": metadata.description,
        "tags": metadata.tags,
        "parameters": parameters,
        "responses": responses,
        "deprecated": metadata.deprecated,
    });
    if let Some(request_body) = &metadata.request_body {
        operation["requestBody"] = request_body.as_json();
    }
    if !security.is_empty() {
        operation["security"] = Value::Array(security);
    }
    operation
}

fn method_name(method: &HttpMethod) -> String {
    if method == HttpMethod::GET {
        "get".to_owned()
    } else if method == HttpMethod::POST {
        "post".to_owned()
    } else if method == HttpMethod::PUT {
        "put".to_owned()
    } else if method == HttpMethod::PATCH {
        "patch".to_owned()
    } else if method == HttpMethod::DELETE {
        "delete".to_owned()
    } else if method == HttpMethod::HEAD {
        "head".to_owned()
    } else if method == HttpMethod::OPTIONS {
        "options".to_owned()
    } else {
        method.as_str().to_ascii_lowercase()
    }
}

fn openapi_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            segment
                .strip_prefix(':')
                .map_or_else(|| segment.to_owned(), |name| format!("{{{name}}}"))
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn path_parameter_names(path: &str) -> impl Iterator<Item = String> + '_ {
    path.split('/').filter_map(|segment| {
        segment
            .strip_prefix('{')
            .and_then(|segment| segment.strip_suffix('}'))
            .map(str::to_owned)
    })
}

fn default_operation_id(handler_name: &str, index: usize) -> String {
    format!("{handler_name}_{index}")
}

pub(crate) fn validate_absolute_path(path: &str) -> Result<(), OpenApiError> {
    if path.starts_with('/') {
        Ok(())
    } else {
        Err(OpenApiError::InvalidPath {
            path: path.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_path_converts_colon_params() {
        assert_eq!(openapi_path("/users/:id/items"), "/users/{id}/items");
        assert_eq!(openapi_path("/:resource/:action"), "/{resource}/{action}");
    }

    #[test]
    fn openapi_path_plain_path_unchanged() {
        assert_eq!(openapi_path("/api/health"), "/api/health");
        assert_eq!(openapi_path("/"), "/");
    }

    #[test]
    fn path_parameter_names_extracts_braced_params() {
        let names: Vec<String> = path_parameter_names("/users/{id}/items/{item_id}").collect();
        assert_eq!(names, vec!["id", "item_id"]);
    }

    #[test]
    fn path_parameter_names_empty_when_none() {
        let names: Vec<String> = path_parameter_names("/api/health").collect();
        assert!(names.is_empty());
    }

    #[test]
    fn method_name_maps_correctly() {
        use ironic_http::HttpMethod;
        assert_eq!(method_name(&HttpMethod::GET), "get");
        assert_eq!(method_name(&HttpMethod::POST), "post");
        assert_eq!(method_name(&HttpMethod::PUT), "put");
        assert_eq!(method_name(&HttpMethod::DELETE), "delete");
        assert_eq!(method_name(&HttpMethod::PATCH), "patch");
        assert_eq!(method_name(&HttpMethod::HEAD), "head");
        assert_eq!(method_name(&HttpMethod::OPTIONS), "options");
    }

    #[test]
    fn default_operation_id_format() {
        assert_eq!(default_operation_id("handler", 0), "handler_0");
        assert_eq!(default_operation_id("get_user", 42), "get_user_42");
    }

    #[test]
    fn validate_absolute_path_accepts_slash() {
        assert!(validate_absolute_path("/openapi.json").is_ok());
        assert!(validate_absolute_path("/").is_ok());
    }

    #[test]
    fn validate_absolute_path_rejects_relative() {
        let err = validate_absolute_path("openapi.json").unwrap_err();
        assert!(matches!(err, OpenApiError::InvalidPath { .. }));
    }

    #[test]
    fn security_scheme_oauth2_json() {
        let mut scopes = BTreeMap::new();
        scopes.insert("read".into(), "Read access".into());
        let scheme = SecurityScheme::OAuth2AuthorizationCode {
            authorization_url: "https://auth.example.com/authorize".into(),
            token_url: "https://auth.example.com/token".into(),
            scopes,
        };
        let json = scheme.as_json();
        assert_eq!(json["type"], "oauth2");
        assert_eq!(
            json["flows"]["authorizationCode"]["scopes"]["read"],
            "Read access"
        );
    }
}
