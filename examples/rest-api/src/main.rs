//! REST API example covering all completed Ironic features:
//!   - Modules, DI, controllers, routes
//!   - Response serialization (`#[derive(Serializable)]`, `#[exclude]`, `#[expose]`)
//!   - API versioning (URI-prefix versioned routes)
//!   - Request body validation (`ValidationPipe` + `garde`)
//!   - Custom exception filters
//!   - Compression (`AxumAdapter::compression()`)
//!   - Security (CORS via `configure_router`)
//!   - Health checks, OpenAPI docs, testing

use std::sync::Arc;

use garde::Validate;
use ironic::prelude::*;
use serde::{Deserialize, Serialize};

/// `#[derive(Serializable)]` reads `#[exclude]` and `#[expose(role = "...")]`
/// attributes and generates a `field_rules()` method returning a `FieldRules`
/// value. Pass that to `SerializeInterceptor` to filter JSON fields at runtime.
#[derive(Clone, Debug, Serialize, Serializable)]
struct ItemView {
    id: u64,
    name: String,
    #[exclude]
    internal_note: String,
    #[expose(role = "admin")]
    admin_only: String,
}

// ---------------------------------------------------------------------------
// Inbound DTO with garde validation attributes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Validate)]
struct CreateItem {
    #[garde(length(min = 1, max = 256))]
    name: String,
    #[garde(range(min = 0.01))]
    price: f64,
}

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

#[derive(Injectable)]
struct CatalogService;

impl CatalogService {
    fn find(&self, id: u64) -> Result<ItemView, HttpError> {
        if id == 404 {
            Err(HttpError::not_found(
                "ITEM_NOT_FOUND",
                "Item does not exist",
            ))
        } else {
            Ok(ItemView {
                id,
                name: format!("item-{id}"),
                internal_note: "internal-only".into(),
                admin_only: "sensitive-data".into(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Controllers
// ---------------------------------------------------------------------------

/// Macro-based v1 controller (no version prefix).
#[controller("/items")]
#[derive(Injectable)]
struct CatalogController {
    service: Arc<CatalogService>,
}

#[routes]
impl CatalogController {
    #[get("/:id")]
    async fn find(&self, #[param] id: u64) -> Result<Json<ItemView>, HttpError> {
        self.service.find(id).map(Json)
    }
}

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

#[derive(Module)]
#[module(providers = [CatalogService], controllers = [CatalogController])]
struct CatalogModule;

#[derive(Module)]
#[module(imports = [CatalogModule, HealthModule])]
struct AppModule;

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(ironic::AxumAdapter::new().compression())
        .build()
        .await
        .expect("application must initialize")
        .listen("127.0.0.1:3000")
        .await
        .expect("application server failed");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use ironic::{
        AxumAdapter, CompiledHttpApplication, ContainerBuilder, FieldRules, FrameworkResponse,
        HttpStatus, SerializeInterceptor, TestApplication, VersionMetadata, VersioningStrategy,
        prelude::*,
    };
    use tower::ServiceExt;

    use super::*;

    struct NotFoundFilter;

    impl ExceptionFilter for NotFoundFilter {
        fn catch(
            &self,
            error: &HttpError,
            _context: &FilterContext,
        ) -> Result<FrameworkResponse, HttpError> {
            if error.status() == HttpStatus::NOT_FOUND {
                Ok(FrameworkResponse::error(
                    HttpStatus::NOT_FOUND,
                    "CUSTOM_NOT_FOUND",
                    format!("Resource not found: {}", error.message()),
                ))
            } else {
                Err(error.clone())
            }
        }
    }

    #[tokio::test]
    async fn v1_found() {
        let app = TestApplication::new::<AppModule>().await.unwrap();
        app.get("/items/7").send().await.assert_json(&ItemView {
            id: 7,
            name: "item-7".to_owned(),
            internal_note: "internal-only".into(),
            admin_only: "sensitive-data".into(),
        });
        app.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn v1_not_found() {
        let app = TestApplication::new::<AppModule>().await.unwrap();
        app.get("/items/404")
            .send()
            .await
            .assert_error("ITEM_NOT_FOUND");
        app.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn health_check() {
        let app = TestApplication::new::<AppModule>().await.unwrap();
        app.get("/health")
            .send()
            .await
            .assert_json(&serde_json::json!({"status": "ok"}));
        app.shutdown().await.unwrap();
    }

    // ------------------------------------------------------------------
    // Builder-based example: v2 versioned route with validation,
    // exception filter, and serialization interceptor
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn versioned_route_with_validation_and_serialization() {
        // Build a POST /v2/items route with validation, versioning,
        // a custom exception filter, and a SerializeInterceptor.

        let route = RouteDefinition::new(
            HttpMethod::POST,
            "/",
            "create",
            handler_fn(
                |_controller: Arc<CatalogController>, mut arguments| async move {
                    let input = arguments.take::<CreateItem>(0)?;
                    Ok(Json(ItemView {
                        id: 42,
                        name: input.name,
                        internal_note: "internal-only".into(),
                        admin_only: "sensitive-data".into(),
                    }))
                },
            ),
        )
        .unwrap()
        .parameter_with_pipe(
            ironic::JsonBody::<CreateItem>::new(),
            Arc::new(ValidationPipe),
        );

        let provider = ProviderDefinition::value(CatalogController {
            service: Arc::new(CatalogService),
        });

        let rules = ironic::FieldRules::new()
            .exclude("internal_note")
            .expose("admin_only", "admin");

        let controller = ControllerDefinition::new::<CatalogController>("/items", provider)
            .unwrap()
            .route(route)
            .version(ironic::VersionMetadata::new(
                "2",
                ironic::VersioningStrategy::Uri,
            ))
            .interceptor(ironic::SerializeInterceptor::new(rules))
            .exception_filter(Arc::new(NotFoundFilter));

        let mut container = ContainerBuilder::new();
        container
            .register(super::CatalogService::provider_definition())
            .unwrap();

        let ct_provider = ProviderDefinition::value(CatalogController {
            service: Arc::new(CatalogService),
        });
        container.register(ct_provider).unwrap();

        let app = Arc::new(ironic::CompiledHttpApplication::new(
            container.build(),
            ironic::compile_controller_routes([controller]).unwrap(),
        ));
        let router = ironic::AxumAdapter::new()
            .compression()
            .build(app)
            .unwrap()
            .into_router();

        // Valid creation without roles — admin_only should be excluded
        let body_bytes = serde_json::to_vec(&CreateItem {
            name: "gadget".into(),
            price: 9.99,
        })
        .unwrap();
        let response = router
            .clone()
            .oneshot(
                axum::http::Request::post("/v2/items")
                    .header("content-type", "application/json")
                    .body(Body::from(body_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), ironic::HttpStatus::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["id"], 42);
        assert_eq!(body["name"], "gadget");
        assert!(!body.as_object().unwrap().contains_key("internal_note"));
        // No admin role set, so admin_only is excluded
        assert!(!body.as_object().unwrap().contains_key("admin_only"));
    }
}
