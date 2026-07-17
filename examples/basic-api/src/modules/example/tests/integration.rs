//! Integration tests for Example — full HTTP request/response cycles.

use ironic::{HttpStatus, TestApplication};
use serde_json::json;

use super::super::*;

async fn app() -> TestApplication {
    TestApplication::new::<ExampleModule>()
        .await
        .expect("test app must initialise")
}

#[ironic::test]
async fn list_returns_ok() {
    let a = app().await;
    assert_eq!(a.get("/example").send().await.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}

#[ironic::test]
async fn create_and_get() {
    let a = app().await;
    let resp = a
        .post("/example")
        .json(&json!({"name": "Test", "description": null}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let id = resp.json::<serde_json::Value>().unwrap()["id"]
        .as_u64()
        .unwrap();
    assert_eq!(
        a.get(&format!("/example/{id}")).send().await.status(),
        HttpStatus::OK
    );
    a.shutdown().await.unwrap();
}

#[ironic::test]
async fn update_works() {
    let a = app().await;
    let id = a
        .post("/example")
        .json(&json!({"name": "Old"}))
        .send()
        .await
        .json::<serde_json::Value>()
        .unwrap()["id"]
        .as_u64()
        .unwrap();
    let resp = a
        .put(&format!("/example/{id}"))
        .json(&json!({"name": "New"}))
        .send()
        .await;
    assert_eq!(resp.json::<serde_json::Value>().unwrap()["name"], "New");
    a.shutdown().await.unwrap();
}

#[ironic::test]
async fn delete_works() {
    let a = app().await;
    let id = a
        .post("/example")
        .json(&json!({"name": "Del"}))
        .send()
        .await
        .json::<serde_json::Value>()
        .unwrap()["id"]
        .as_u64()
        .unwrap();
    a.delete(&format!("/example/{id}")).send().await;
    assert_eq!(
        a.get(&format!("/example/{id}")).send().await.status(),
        HttpStatus::NOT_FOUND
    );
    a.shutdown().await.unwrap();
}

#[ironic::test]
async fn not_found_returns_404() {
    let a = app().await;
    a.get("/example/999").send().await.assert_status(404);
    a.shutdown().await.unwrap();
}

// To enable request body validation, wire ValidationPipe in your controller:
//   #[controller("/example")]
//   #[pipe(ValidationPipe)]
// The CreateExampleDto already has garde validation rules defined.
