//! Integration tests for full auth flow.

use super::super::*;
use ironic::{HttpStatus, TestApplication};
use serde_json::json;

async fn app() -> TestApplication {
    TestApplication::new::<AuthModule>().await.unwrap()
}

#[tokio::test]
async fn register_and_login_flow() {
    let a = app().await;
    let resp = a
        .post("/auth/register")
        .json(&json!({"email":"flow@test.com","password":"pass123","name":"Flow"}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::OK);

    let resp = a
        .post("/auth/login")
        .json(&json!({"email":"flow@test.com","password":"pass123"}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let tokens: serde_json::Value = resp.json().unwrap();
    assert!(tokens["access_token"].as_str().unwrap().len() > 10);
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn login_invalid_credentials() {
    let a = app().await;
    let resp = a
        .post("/auth/login")
        .json(&json!({"email":"nobody@test.com","password":"wrong"}))
        .send()
        .await;
    assert_eq!(resp.status(), HttpStatus::UNAUTHORIZED);
    a.shutdown().await.unwrap();
}
