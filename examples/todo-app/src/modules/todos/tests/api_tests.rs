//! Integration tests for Todo API — requires a running PostgreSQL database.
//!
//! Run with: DATABASE_URL=postgres://user:password@localhost:5432/todo_example_test cargo test -- --ignored

use ironic::{HttpStatus, TestApplication};
use serde_json::json;

use crate::modules::todos::TodosModule;

async fn app() -> TestApplication {
    TestApplication::new::<TodosModule>()
        .await
        .expect("test app must initialise")
}

#[ignore]
#[tokio::test]
async fn list_returns_ok() {
    let a = app().await;
    let resp = a.get("/api/todos").send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn create_returns_todo() {
    let a = app().await;
    let resp = a
        .post("/api/todos")
        .json(&json!({"title": "Write docs", "description": "Document the API"}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let todo = resp.json::<serde_json::Value>().unwrap();
    assert_eq!(todo["title"], "Write docs");
    assert!(todo["id"].is_string());
    a.shutdown().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn create_rejects_empty_title() {
    let a = app().await;
    let resp = a
        .post("/api/todos")
        .json(&json!({"title": ""}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::BAD_REQUEST);
    a.shutdown().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn get_returns_404_for_missing() {
    let a = app().await;
    a.get("/api/todos/00000000-0000-0000-0000-000000000000")
        .send()
        .await
        .assert_status(404);
    a.shutdown().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn lifecycle_create_update_toggle_delete() {
    let a = app().await;

    let created = a
        .post("/api/todos")
        .json(&json!({"title": "Learn Ironic"}))
        .send()
        .await
        .json::<serde_json::Value>()
        .unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["completed"], false);

    let updated = a
        .put(format!("/api/todos/{id}"))
        .json(&json!({"title": "Master Ironic"}))
        .send()
        .await
        .json::<serde_json::Value>()
        .unwrap();
    assert_eq!(updated["title"], "Master Ironic");

    let toggled = a
        .post(format!("/api/todos/{id}/toggle"))
        .send()
        .await
        .json::<serde_json::Value>()
        .unwrap();
    assert_eq!(toggled["completed"], true);

    let del_resp = a.delete(format!("/api/todos/{id}")).send().await;
    assert_eq!(del_resp.status(), HttpStatus::OK);

    a.shutdown().await.unwrap();
}
