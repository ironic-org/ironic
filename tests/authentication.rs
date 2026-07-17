//! Feature-level contracts for authentication and authorization helpers.

#![cfg(feature = "auth")]

use std::collections::HashSet;

use ironic::{
    Request, Guard, HeaderMap, HeaderValue, HttpMethod, RequestContext,
    auth::{
        AuthContext, Authorizable, Principal, RequireAccess, bearer_token, hash_password,
        verify_password,
    },
};

#[derive(Clone, Debug)]
struct User {
    id: String,
    roles: HashSet<String>,
}

impl Principal for User {
    fn subject(&self) -> &str {
        &self.id
    }
}

impl Authorizable for User {
    fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    fn has_permission(&self, _permission: &str) -> bool {
        false
    }
}

fn request_with_authorization(value: &str) -> Request {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        HeaderValue::from_str(value).unwrap(),
    );
    Request::new(HttpMethod::GET, "/".parse().unwrap(), headers, Vec::new())
}

#[test]
fn bearer_credentials_are_strictly_parsed() {
    let request = request_with_authorization("Bearer signed-token");
    assert_eq!(bearer_token(&request).unwrap(), Some("signed-token"));

    let malformed = request_with_authorization("Basic credential");
    assert!(bearer_token(&malformed).is_err());
}

#[test]
fn passwords_are_hashed_with_unique_salts_and_verified() {
    let first = hash_password(b"correct horse battery staple").unwrap();
    let second = hash_password(b"correct horse battery staple").unwrap();
    assert_ne!(first, second);
    assert!(verify_password(b"correct horse battery staple", &first).unwrap());
    assert!(!verify_password(b"wrong", &first).unwrap());
}

#[tokio::test]
async fn role_guards_use_request_authentication_state() {
    let mut context = RequestContext::new(Request::new(
        HttpMethod::GET,
        "/".parse().unwrap(),
        HeaderMap::new(),
        Vec::new(),
    ));
    context.insert_extension(AuthContext::new(Some(User {
        id: "user-1".into(),
        roles: HashSet::from(["admin".into()]),
    })));

    assert!(
        RequireAccess::<User>::role("admin")
            .can_activate(&mut context)
            .await
            .is_ok()
    );
    assert!(
        RequireAccess::<User>::role("operator")
            .can_activate(&mut context)
            .await
            .is_err()
    );
}

#[cfg(feature = "jwt")]
#[test]
fn jwt_service_signs_and_validates_claims() {
    use ironic::auth::jwt::{JwtService, driver::Algorithm};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Claims {
        sub: String,
        exp: u64,
    }

    let service = JwtService::hmac(
        b"a-test-secret-that-is-not-for-production",
        Algorithm::HS256,
    );
    let claims = Claims {
        sub: "user-1".into(),
        exp: 4_102_444_800,
    };
    let token = service.encode(&claims).unwrap();
    assert_eq!(
        service.decode::<Claims>(&token).unwrap().claims.sub,
        "user-1"
    );
}

#[cfg(feature = "oauth")]
#[test]
fn oauth_authorization_requests_include_state_and_pkce() {
    use ironic::auth::oauth::{authorization_request, basic_client};

    let client = basic_client(
        "client-id",
        Some("client-secret".into()),
        "https://identity.example/authorize",
        "https://identity.example/token",
        "https://app.example/callback",
    )
    .unwrap();
    let request = authorization_request(&client, ["openid".to_owned(), "profile".to_owned()]);
    let query = request.url.query().unwrap();
    assert!(query.contains("state="));
    assert!(query.contains("code_challenge="));
    assert!(query.contains("scope=openid"));
}

#[cfg(feature = "sessions")]
#[tokio::test]
async fn in_memory_sessions_round_trip_and_delete() {
    use std::time::Duration;

    use ironic::auth::sessions::{InMemorySessionStore, Session, SessionStore};

    let store = InMemorySessionStore::default();
    let mut session = Session::new(Duration::from_mins(1)).unwrap();
    session.insert("user_id", "user-1").unwrap();
    let id = session.id.clone();
    store.save(session).await.unwrap();

    let loaded = store.load(&id).await.unwrap().unwrap();
    assert_eq!(
        loaded.get::<String>("user_id").unwrap().as_deref(),
        Some("user-1")
    );
    store.delete(&id).await.unwrap();
    assert!(store.load(&id).await.unwrap().is_none());
}
