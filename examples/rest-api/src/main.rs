//! REST API example covering modules, DI, validation, errors, health, and testing.

use std::sync::Arc;

use rustframe::prelude::*;
use rustframe_openapi::{OpenApiAxumExt, OpenApiConfig, OpenApiSchema};
use rustframe_platform_axum::AxumAdapter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, OpenApiSchema, PartialEq, Serialize)]
struct ItemView {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize, OpenApiSchema, Serialize)]
struct CreateItem {
    name: String,
}

#[derive(Injectable)]
struct CatalogService;

impl CatalogService {
    #[allow(clippy::unused_self)]
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
            })
        }
    }
}

#[controller("/items")]
#[derive(Injectable)]
struct CatalogController {
    service: Arc<CatalogService>,
}

#[routes]
impl CatalogController {
    #[get("/:id")]
    #[allow(clippy::unused_async)]
    async fn find(&self, #[param] id: u64) -> Result<Json<ItemView>, HttpError> {
        if id == 0 {
            return Err(HttpError::unprocessable_entity(
                "INVALID_ITEM_ID",
                "Item ID must be greater than zero",
            ));
        }
        self.service.find(id).map(Json)
    }

    #[post("/")]
    #[allow(clippy::unused_async)]
    async fn create(&self, #[body] input: CreateItem) -> Result<Json<ItemView>, HttpError> {
        if input.name.trim().is_empty() {
            return Err(HttpError::unprocessable_entity(
                "INVALID_ITEM_NAME",
                "Item name cannot be empty",
            ));
        }
        Ok(Json(ItemView {
            id: 1,
            name: input.name,
        }))
    }
}

#[derive(Module)]
#[module(providers = [CatalogService], controllers = [CatalogController])]
struct CatalogModule;

#[derive(Module)]
#[module(imports = [CatalogModule, HealthModule])]
struct AppModule;

#[rustframe::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(
            AxumAdapter::new()
                .with_openapi(
                    OpenApiConfig::new("RustFrame REST API", "1.0.0")
                        .description("Items API example")
                        .schema::<CreateItem>("CreateItem")
                        .schema::<ItemView>("ItemView"),
                )
                .swagger_ui("/docs"),
        )
        .build()
        .await
        .expect("application must initialize")
        .listen("127.0.0.1:3000")
        .await
        .expect("application server failed");
}

#[cfg(test)]
mod tests {
    use rustframe_testing::TestApplication;

    use super::*;

    #[tokio::test]
    async fn exercises_success_validation_errors_and_health() {
        let application = TestApplication::new::<AppModule>().await.unwrap();

        application
            .get("/items/7")
            .send()
            .await
            .assert_json(&ItemView {
                id: 7,
                name: "item-7".to_owned(),
            });
        application
            .get("/items/0")
            .send()
            .await
            .assert_error("INVALID_ITEM_ID");
        application
            .get("/items/404")
            .send()
            .await
            .assert_error("ITEM_NOT_FOUND");
        application
            .post("/items")
            .json(&CreateItem {
                name: String::new(),
            })
            .send()
            .await
            .assert_error("INVALID_ITEM_NAME");
        application
            .get("/health")
            .send()
            .await
            .assert_json(&serde_json::json!({"status": "ok"}));

        application.shutdown().await.unwrap();
    }
}
