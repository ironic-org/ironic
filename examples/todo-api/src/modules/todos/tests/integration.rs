//! Integration tests — full HTTP request/response cycles via TestApplication.

use ironic::{HttpStatus, TestApplication};

use super::super::*;

async fn app() -> TestApplication {
    TestApplication::new::<TodosModule>()
        .await
        .expect("test application must initialise")
}

#[tokio::test]
async fn list_empty_returns_ok() {
    let app = app().await;
    let response = app.get("/todos").send().await;
    assert_eq!(response.status(), HttpStatus::OK);
    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn create_and_get_todo() {
    let app = app().await;

    let resp = app
        .post("/todos")
        .json(&serde_json::json!({"title": "Learn Ironic", "priority": "high"}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let created: serde_json::Value = resp.json().unwrap();

    let id = created["id"].as_u64().unwrap();
    let get = app.get(&format!("/todos/{id}")).send().await;
    assert_eq!(get.status(), HttpStatus::OK);
    let fetched: serde_json::Value = get.json().unwrap();
    assert_eq!(fetched["title"], "Learn Ironic");

    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn update_todo() {
    let app = app().await;

    let resp = app
        .post("/todos")
        .json(&serde_json::json!({"title": "Old Title"}))
        .send()
        .await;
    let id = resp
        .json::<serde_json::Value>()
        .unwrap()["id"]
        .as_u64()
        .unwrap();

    let update = app
        .put(&format!("/todos/{id}"))
        .json(&serde_json::json!({"title": "New Title", "completed": true}))
        .send()
        .await;
    let updated: serde_json::Value = update.json().unwrap();
    assert_eq!(updated["title"], "New Title");
    assert_eq!(updated["completed"], true);

    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn delete_todo() {
    let app = app().await;

    let resp = app
        .post("/todos")
        .json(&serde_json::json!({"title": "Delete me"}))
        .send()
        .await;
    let id = resp
        .json::<serde_json::Value>()
        .unwrap()["id"]
        .as_u64()
        .unwrap();

    let del = app.delete(&format!("/todos/{id}")).send().await;
    assert_eq!(del.status(), HttpStatus::NO_CONTENT);

    let get = app.get(&format!("/todos/{id}")).send().await;
    assert_eq!(get.status(), HttpStatus::NOT_FOUND);

    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn get_not_found_returns_404() {
    let app = app().await;
    app.get("/todos/999").send().await.assert_status(404);
    app.shutdown().await.unwrap();
}
