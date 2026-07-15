//! Unit tests for the auth module.

use crate::modules::auth::dto::{LoginDto, RegisterDto};
use crate::modules::auth::guards::auth_guard::AuthGuard;
use crate::modules::auth::guards::role_guard::RoleGuard;
use crate::modules::auth::services::auth_service::AuthService;
use crate::modules::auth::services::password_service::PasswordService;
use ironic::{FrameworkRequest, Guard, GuardDecision, RequestContext};
use std::sync::Arc;

// ── PasswordService ──────────────────────────────────────────────

#[test]
fn hash_and_verify() {
    let svc = PasswordService;
    let hash = svc.hash("password123").unwrap();
    assert!(svc.verify("password123", &hash).unwrap());
    assert!(!svc.verify("wrong", &hash).unwrap());
}

#[test]
fn unique_salts() {
    let svc = PasswordService;
    let h1 = svc.hash("password123").unwrap();
    let h2 = svc.hash("password123").unwrap();
    assert_ne!(h1, h2);
}

// ── AuthService ───────────────────────────────────────────────────

#[test]
fn register_and_login() {
    let svc = AuthService {
        password: Arc::new(PasswordService),
    };
    let user = svc
        .register(RegisterDto {
            email: "test@test.com".into(),
            password: "pass123".into(),
            name: "Test".into(),
        })
        .unwrap();
    assert_eq!(user.email, "test@test.com");
    let tokens = svc
        .login(LoginDto {
            email: "test@test.com".into(),
            password: "pass123".into(),
        })
        .unwrap();
    assert!(!tokens.access_token.is_empty());
}

#[test]
fn duplicate_email_rejected() {
    let svc = AuthService {
        password: Arc::new(PasswordService),
    };
    svc.register(RegisterDto {
        email: "dup@test.com".into(),
        password: "pass".into(),
        name: "A".into(),
    })
    .unwrap();
    assert!(
        svc.register(RegisterDto {
            email: "dup@test.com".into(),
            password: "pass".into(),
            name: "B".into()
        })
        .is_err()
    );
}

#[test]
fn wrong_password_rejected() {
    let svc = AuthService {
        password: Arc::new(PasswordService),
    };
    svc.register(RegisterDto {
        email: "x@test.com".into(),
        password: "correct".into(),
        name: "X".into(),
    })
    .unwrap();
    assert!(
        svc.login(LoginDto {
            email: "x@test.com".into(),
            password: "wrong".into()
        })
        .is_err()
    );
}

// ── Guards ────────────────────────────────────────────────────────

#[tokio::test]
async fn auth_guard_denies_missing_header() {
    let mut ctx = RequestContext::new(FrameworkRequest::new(
        ironic::HttpMethod::GET,
        "/".parse().unwrap(),
        ironic::HeaderMap::new(),
        vec![],
    ));
    let decision = AuthGuard.can_activate(&mut ctx).await.unwrap();
    assert!(matches!(decision, GuardDecision::Deny));
}

#[tokio::test]
async fn role_guard_denies_wrong_role() {
    let mut ctx = RequestContext::new(FrameworkRequest::new(
        ironic::HttpMethod::GET,
        "/".parse().unwrap(),
        ironic::HeaderMap::new(),
        vec![],
    ));
    ctx.insert_extension("user".to_string());
    let guard = RoleGuard::new(&["admin"]);
    let decision = guard.can_activate(&mut ctx).await.unwrap();
    assert!(matches!(decision, GuardDecision::Deny));
}
