//! Macro-based Ironic application with a `GET /users/:id` route.

use std::sync::Arc;

use ironic::{AxumAdapter, AxumApplication, prelude::*};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct UserView {
    id: u64,
    name: &'static str,
}

#[derive(Injectable)]
struct UsersService;

impl UsersService {
    #[allow(clippy::unused_self)]
    fn find(&self, id: u64) -> Result<UserView, HttpError> {
        if id == 1 {
            Ok(UserView { id, name: "Ada" })
        } else {
            Err(HttpError::not_found(
                "USER_NOT_FOUND",
                "The requested user does not exist",
            ))
        }
    }
}

#[controller("/users")]
#[derive(Injectable)]
struct UsersController {
    users: Arc<UsersService>,
}

#[routes]
impl UsersController {
    #[get("/:id")]
    #[allow(clippy::unused_async)]
    async fn find_one(&self, #[param] id: u64) -> Result<Json<UserView>, HttpError> {
        self.users.find(id).map(Json)
    }
}

#[derive(Module)]
#[module(providers = [UsersService], controllers = [UsersController])]
struct UsersModule;

#[derive(Module)]
#[module(imports = [UsersModule])]
struct AppModule;

#[ironic::main]
async fn main() {
    let _application = application().await;

    println!("Ironic route ready: GET /users/:id");
}

async fn application() -> FrameworkApplication<AxumApplication> {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must compile and initialize")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    use super::*;
    use ironic::HttpStatus;

    #[test]
    fn macro_application_matches_the_explicit_route_behavior() {
        ironic::__private::block_on(async {
            let application = application().await;

            let response = application
                .platform()
                .router()
                .clone()
                .oneshot(Request::get("/users/1").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), HttpStatus::OK);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
                serde_json::json!({"id": 1, "name": "Ada"})
            );

            let response = application
                .platform()
                .router()
                .clone()
                .oneshot(Request::get("/users/99").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), HttpStatus::NOT_FOUND);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
                "USER_NOT_FOUND"
            );
        });
    }
}
