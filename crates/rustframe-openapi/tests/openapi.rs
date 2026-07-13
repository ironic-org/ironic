//! `OpenAPI` document generation and Axum endpoint integration tests.

use std::{collections::BTreeMap, sync::Arc};

use axum::{body::Body, body::to_bytes, http::Request};
use rustframe_di::{ContainerBuilder, ProviderDefinition, Scope};
use rustframe_http::{
    CompiledHttpApplication, ControllerDefinition, HttpError, HttpMethod, RouteDefinition,
    compile_controller_routes, handler_fn,
};
use rustframe_openapi::{
    OpenApiAxumError, OpenApiAxumExt, OpenApiConfig, OpenApiDocument, OpenApiError,
    OpenApiOperation, OpenApiParameter, OpenApiRequestBody, OpenApiResponse, OpenApiRouteExt,
    OpenApiSchema, ParameterLocation, SecurityScheme,
};
use rustframe_platform::HttpPlatformAdapter;
use rustframe_platform_axum::AxumAdapter;
use serde::Serialize;
use tower::ServiceExt;

#[derive(OpenApiSchema)]
#[allow(dead_code)]
struct CreateUser {
    #[serde(rename = "displayName")]
    name: String,
    age: Option<u16>,
}

#[derive(OpenApiSchema, Serialize)]
struct UserView {
    id: u64,
    name: String,
}

struct UsersController;

fn application() -> Arc<CompiledHttpApplication> {
    let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
        Ok(UsersController)
    });
    let operation = OpenApiOperation::new()
        .summary("Find a user")
        .operation_id("findUser")
        .tag("users")
        .parameter(
            OpenApiParameter::new::<u64>("id", ParameterLocation::Path)
                .description("User identifier"),
        )
        .request_body(OpenApiRequestBody::json::<CreateUser>().optional())
        .response(
            "200",
            OpenApiResponse::new("User found")
                .json::<UserView>()
                .example(serde_json::json!({"id": 7, "name": "Ada"})),
        )
        .security("bearer", std::iter::empty::<String>());
    let route = RouteDefinition::new(
        HttpMethod::GET,
        "/:id",
        "find",
        handler_fn(|_controller: Arc<UsersController>, _arguments| async {
            Ok::<_, HttpError>("ok")
        }),
    )
    .unwrap()
    .openapi(operation);
    let controller = ControllerDefinition::new::<UsersController>("/users", provider)
        .unwrap()
        .route(route);
    let mut container = ContainerBuilder::new();
    container.register(controller.provider().clone()).unwrap();
    Arc::new(CompiledHttpApplication::new(
        container.build(),
        compile_controller_routes([controller]).unwrap(),
    ))
}

fn config() -> OpenApiConfig {
    OpenApiConfig::new("Users API", "1.0.0")
        .description("Example API")
        .schema::<CreateUser>("CreateUser")
        .security_scheme(
            "bearer",
            SecurityScheme::HttpBearer {
                bearer_format: Some("JWT".to_owned()),
            },
        )
        .security_scheme(
            "oauth",
            SecurityScheme::OAuth2AuthorizationCode {
                authorization_url: "https://example.test/authorize".to_owned(),
                token_url: "https://example.test/token".to_owned(),
                scopes: BTreeMap::from([("users:read".to_owned(), "Read users".to_owned())]),
            },
        )
}

fn application_with_routes(routes: Vec<RouteDefinition>) -> Arc<CompiledHttpApplication> {
    let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
        Ok(UsersController)
    });
    let controller = ControllerDefinition::new::<UsersController>("/", provider)
        .unwrap()
        .with_routes(routes);
    let mut container = ContainerBuilder::new();
    container.register(controller.provider().clone()).unwrap();
    Arc::new(CompiledHttpApplication::new(
        container.build(),
        compile_controller_routes([controller]).unwrap(),
    ))
}

fn empty_route(path: &str, handler_name: &'static str) -> RouteDefinition {
    RouteDefinition::new(
        HttpMethod::GET,
        path,
        handler_name,
        handler_fn(|_controller: Arc<UsersController>, _arguments| async {
            Ok::<_, HttpError>("ok")
        }),
    )
    .unwrap()
}

#[test]
fn derives_schemas_and_discovers_route_metadata() {
    let schema = CreateUser::openapi_schema();
    assert_eq!(schema["properties"]["displayName"]["type"], "string");
    assert_eq!(schema["properties"]["age"]["nullable"], true);
    assert_eq!(schema["required"], serde_json::json!(["displayName"]));

    let document = OpenApiDocument::from_application(&application(), &config()).unwrap();
    let operation = &document.as_value()["paths"]["/users/{id}"]["get"];
    assert_eq!(document.as_value()["openapi"], "3.1.0");
    assert_eq!(operation["operationId"], "findUser");
    assert_eq!(operation["tags"], serde_json::json!(["users"]));
    assert_eq!(operation["parameters"][0]["name"], "id");
    assert_eq!(operation["responses"]["200"]["description"], "User found");
    assert_eq!(operation["security"][0]["bearer"], serde_json::json!([]));
    assert_eq!(
        document.as_value()["components"]["securitySchemes"]["bearer"]["scheme"],
        "bearer"
    );
}

#[tokio::test]
async fn serves_json_and_swagger_ui_from_the_axum_wrapper() {
    let router = AxumAdapter::new()
        .with_openapi(config())
        .swagger_ui("/docs")
        .build(application())
        .unwrap()
        .into_router();

    let response = router
        .clone()
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let document: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(document["info"]["title"], "Users API");

    let response = router
        .oneshot(Request::get("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("SwaggerUIBundle"));
    assert!(html.contains("/openapi.json"));
}

#[test]
fn rejects_duplicate_operation_ids_and_generated_endpoint_conflicts() {
    let duplicate = OpenApiOperation::new().operation_id("duplicate");
    let application = application_with_routes(vec![
        empty_route("/first", "first").openapi(duplicate.clone()),
        empty_route("/second", "second").openapi(duplicate),
    ]);
    assert!(matches!(
        OpenApiDocument::from_application(&application, &config()),
        Err(OpenApiError::DuplicateOperationId { .. })
    ));

    let application = application_with_routes(vec![empty_route("/openapi.json", "reserved")]);
    assert!(matches!(
        AxumAdapter::new().with_openapi(config()).build(application),
        Err(OpenApiAxumError::OpenApi(
            OpenApiError::EndpointConflict { .. }
        ))
    ));
}
