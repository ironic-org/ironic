//! Authentication, password hashing, and authorization pipeline helpers.

use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    Guard, GuardDecision, GuardFuture, HttpError, Middleware, MiddlewareNext, PipelineFuture,
    Request, RequestContext,
};

/// The upstream Argon2 password-hashing API.
pub use ::argon2 as password_driver;

#[cfg(feature = "jwt")]
pub mod jwt;
#[cfg(feature = "oauth")]
pub mod oauth;
#[cfg(feature = "sessions")]
pub mod sessions;

/// A boxed asynchronous authentication operation.
///
/// Returned by [`Authenticator::authenticate`].
pub type AuthenticationFuture<'a, P> =
    Pin<Box<dyn Future<Output = Result<Option<P>, AuthError>> + Send + 'a>>;

/// A safe authentication or authorization failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
    /// Presented credentials are absent or invalid.
    #[error("IRONIC_AUTH_UNAUTHORIZED: {0}")]
    Unauthorized(String),
    /// An authenticated principal lacks the required access.
    #[error("IRONIC_AUTH_FORBIDDEN: {0}")]
    Forbidden(String),
    /// Authentication was configured incorrectly.
    #[error("IRONIC_AUTH_CONFIGURATION: {0}")]
    Configuration(String),
}

impl AuthError {
    fn into_http_error(self) -> HttpError {
        match self {
            Self::Unauthorized(message) => {
                HttpError::unauthorized("IRONIC_AUTH_UNAUTHORIZED", message)
            }
            Self::Forbidden(message) => HttpError::forbidden("IRONIC_AUTH_FORBIDDEN", message),
            Self::Configuration(message) => {
                HttpError::internal("IRONIC_AUTH_CONFIGURATION", message)
            }
        }
    }
}

/// An application-defined authenticated identity.
///
/// # Example
///
/// ```rust
/// use ironic::auth::{Principal, Authorizable};
///
/// struct User {
///     id: String,
///     roles: Vec<String>,
/// }
///
/// impl Principal for User {
///     fn subject(&self) -> &str {
///         &self.id
///     }
/// }
///
/// impl Authorizable for User {
///     fn has_role(&self, role: &str) -> bool {
///         self.roles.iter().any(|r| r == role)
///     }
///     fn has_permission(&self, _permission: &str) -> bool {
///         false
///     }
/// }
/// ```
pub trait Principal: Send + Sync + 'static {
    /// Returns the stable subject identifier for logs and authorization decisions.
    fn subject(&self) -> &str;
}

/// Role and permission information exposed by a principal.
pub trait Authorizable: Principal {
    /// Returns whether this principal has `role`.
    fn has_role(&self, role: &str) -> bool;

    /// Returns whether this principal has `permission`.
    fn has_permission(&self, permission: &str) -> bool;
}

/// Authentication state attached to one request.
#[derive(Debug)]
pub struct AuthContext<P> {
    principal: Option<Arc<P>>,
}

impl<P> Clone for AuthContext<P> {
    fn clone(&self) -> Self {
        Self {
            principal: self.principal.clone(),
        }
    }
}

impl<P> AuthContext<P> {
    /// Creates authentication state for an optional principal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ironic::auth::AuthContext;
    ///
    /// let ctx = AuthContext::new(Some("user-42"));
    /// assert!(ctx.is_authenticated());
    /// assert_eq!(ctx.principal(), Some(&"user-42"));
    /// ```
    #[must_use]
    pub fn new(principal: Option<P>) -> Self {
        Self {
            principal: principal.map(Arc::new),
        }
    }

    /// Returns the authenticated principal, if credentials were accepted.
    #[must_use]
    pub fn principal(&self) -> Option<&P> {
        self.principal.as_deref()
    }

    /// Returns whether the request has an authenticated principal.
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.principal.is_some()
    }
}

/// Converts request credentials into an application principal.
pub trait Authenticator<P>: Send + Sync + 'static {
    /// Authenticates one request. `Ok(None)` represents an anonymous request.
    fn authenticate<'a>(&'a self, request: &'a Request) -> AuthenticationFuture<'a, P>;
}

/// Middleware that authenticates a request and stores [`AuthContext`].
///
/// # Type parameters
///
/// * `A` — An [`Authenticator`] implementation that extracts credentials.
/// * `P` — The [`Principal`] type produced by the authenticator.
pub struct AuthenticationMiddleware<A, P> {
    authenticator: A,
    marker: std::marker::PhantomData<fn() -> P>,
}

impl<A, P> AuthenticationMiddleware<A, P> {
    /// Creates middleware backed by `authenticator`.
    #[must_use]
    pub const fn new(authenticator: A) -> Self {
        Self {
            authenticator,
            marker: std::marker::PhantomData,
        }
    }
}

impl<A, P> Middleware for AuthenticationMiddleware<A, P>
where
    A: Authenticator<P>,
    P: Send + Sync + 'static,
{
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let principal = self
                .authenticator
                .authenticate(context.request())
                .await
                .map_err(AuthError::into_http_error)?;
            context.insert_extension(AuthContext::new(principal));
            next.run(context).await
        })
    }
}

/// Guard that rejects anonymous requests.
///
/// # Type parameters
///
/// * `P` — The [`Principal`] type required for this route.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequireAuthenticated<P>(std::marker::PhantomData<fn() -> P>);

impl<P> RequireAuthenticated<P> {
    /// Creates an authentication guard for principal type `P`.
    #[must_use]
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<P: Send + Sync + 'static> Guard for RequireAuthenticated<P> {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        let allowed = context
            .extension::<AuthContext<P>>()
            .is_some_and(AuthContext::is_authenticated);
        Box::pin(async move {
            if allowed {
                Ok(GuardDecision::Allow)
            } else {
                Err(HttpError::unauthorized(
                    "IRONIC_AUTH_REQUIRED",
                    "Authentication is required",
                ))
            }
        })
    }
}

/// A role-or-permission authorization requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessRequirement {
    /// Require an application role.
    Role(String),
    /// Require an application permission.
    Permission(String),
}

/// Guard enforcing a role or permission on an authenticated principal.
///
/// Used with [`RequireAccess::role`] or [`RequireAccess::permission`].
///
/// # Type parameters
///
/// * `P` — The [`Authorizable`] principal type.
#[derive(Clone, Debug)]
pub struct RequireAccess<P> {
    requirement: AccessRequirement,
    marker: std::marker::PhantomData<fn() -> P>,
}

impl<P> RequireAccess<P> {
    /// Requires `role`.
    #[must_use]
    pub fn role(role: impl Into<String>) -> Self {
        Self {
            requirement: AccessRequirement::Role(role.into()),
            marker: std::marker::PhantomData,
        }
    }

    /// Requires `permission`.
    #[must_use]
    pub fn permission(permission: impl Into<String>) -> Self {
        Self {
            requirement: AccessRequirement::Permission(permission.into()),
            marker: std::marker::PhantomData,
        }
    }
}

impl<P: Authorizable> Guard for RequireAccess<P> {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        let decision = context
            .extension::<AuthContext<P>>()
            .and_then(AuthContext::principal)
            .map(|principal| match &self.requirement {
                AccessRequirement::Role(role) => principal.has_role(role),
                AccessRequirement::Permission(permission) => principal.has_permission(permission),
            });
        Box::pin(async move {
            match decision {
                Some(true) => Ok(GuardDecision::Allow),
                Some(false) => Err(HttpError::forbidden(
                    "IRONIC_AUTH_ACCESS_DENIED",
                    "The authenticated principal lacks the required access",
                )),
                None => Err(HttpError::unauthorized(
                    "IRONIC_AUTH_REQUIRED",
                    "Authentication is required",
                )),
            }
        })
    }
}

/// Extracts an RFC 6750 bearer credential from request headers.
///
/// Returns `Ok(None)` when no `Authorization` header is present.
///
/// # Errors
///
/// Returns [`AuthError::Unauthorized`] when the `Authorization` header
/// is not valid UTF-8, does not use the `Bearer` scheme, or contains
/// multiple credentials.
pub fn bearer_token(request: &Request) -> Result<Option<&str>, AuthError> {
    let Some(value) = request.headers().get(http::header::AUTHORIZATION) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| AuthError::Unauthorized("Authorization header is not valid text".into()))?;
    let Some((scheme, credential)) = value.split_once(' ') else {
        return Err(AuthError::Unauthorized(
            "Authorization header must use the Bearer scheme".into(),
        ));
    };
    if !scheme.eq_ignore_ascii_case("bearer") || credential.is_empty() || credential.contains(' ') {
        return Err(AuthError::Unauthorized(
            "Authorization header must contain one Bearer credential".into(),
        ));
    }
    Ok(Some(credential))
}

/// Hashes a password with Argon2id and a cryptographically random salt.
///
/// The returned string contains all parameters needed for verification
/// and can be passed directly to [`verify_password`].
///
/// # Example
///
/// ```rust
/// use ironic::auth::{hash_password, verify_password};
///
/// let hash = hash_password(b"my-secure-password").unwrap();
/// assert!(verify_password(b"my-secure-password", &hash).unwrap());
/// assert!(!verify_password(b"wrong-password", &hash).unwrap());
/// ```
///
/// # Errors
///
/// Returns the upstream password-hash error if hashing fails.
pub fn hash_password(password: &[u8]) -> Result<String, password_driver::password_hash::Error> {
    use password_driver::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};

    let salt = SaltString::generate(&mut OsRng);
    password_driver::Argon2::default()
        .hash_password(password, &salt)
        .map(|hash| hash.to_string())
}

/// Verifies a password against an encoded Argon2 password hash.
///
/// # Example
///
/// ```rust
/// use ironic::auth::{hash_password, verify_password};
///
/// let hash = hash_password(b"password").unwrap();
/// assert!(verify_password(b"password", &hash).unwrap());
/// ```
///
/// # Errors
///
/// Returns an error when the encoded hash is malformed. A password mismatch returns `Ok(false)`.
pub fn verify_password(
    password: &[u8],
    encoded_hash: &str,
) -> Result<bool, password_driver::password_hash::Error> {
    use password_driver::password_hash::{Error, PasswordHash, PasswordVerifier};

    let parsed = PasswordHash::new(encoded_hash)?;
    match password_driver::Argon2::default().verify_password(password, &parsed) {
        Ok(()) => Ok(true),
        Err(Error::Password) => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HeaderMap, HttpError, HttpMethod, HttpStatus, Request, Uri};

    fn request_with_auth_header(value: &str) -> Request {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            value.parse().unwrap(),
        );
        Request::new(HttpMethod::GET, "/".parse::<Uri>().unwrap(), headers, Vec::new())
    }

    // -----------------------------------------------------------------------
    // bearer_token
    // -----------------------------------------------------------------------

    #[test]
    fn bearer_token_returns_none_when_missing() {
        let request = Request::new(
            HttpMethod::GET,
            "/".parse::<Uri>().unwrap(),
            HeaderMap::new(),
            Vec::new(),
        );
        assert_eq!(bearer_token(&request).unwrap(), None);
    }

    #[test]
    fn bearer_token_returns_credential() {
        let request = request_with_auth_header("Bearer my-token");
        assert_eq!(bearer_token(&request).unwrap(), Some("my-token"));
    }

    #[test]
    fn bearer_token_rejects_wrong_scheme() {
        let request = request_with_auth_header("Basic dXNlcjpwYXNz");
        assert!(bearer_token(&request).is_err());
    }

    #[test]
    fn bearer_token_rejects_no_space() {
        let request = request_with_auth_header("Bearernospace");
        assert!(bearer_token(&request).is_err());
    }

    #[test]
    fn bearer_token_rejects_empty_credential() {
        let request = request_with_auth_header("Bearer ");
        assert!(bearer_token(&request).is_err());
    }

    #[test]
    fn bearer_token_case_insensitive_scheme() {
        let request = request_with_auth_header("BEARER token-value");
        assert_eq!(bearer_token(&request).unwrap(), Some("token-value"));
    }

    // -----------------------------------------------------------------------
    // hash_password / verify_password
    // -----------------------------------------------------------------------

    #[test]
    fn hash_and_verify_round_trip() {
        let hash = hash_password(b"hello-world").unwrap();
        assert!(verify_password(b"hello-world", &hash).unwrap());
    }

    #[test]
    fn hash_verify_wrong_password() {
        let hash = hash_password(b"correct").unwrap();
        assert!(!verify_password(b"wrong", &hash).unwrap());
    }

    #[test]
    fn hash_verify_malformed_hash() {
        assert!(verify_password(b"password", "not-a-valid-hash").is_err());
    }

    // -----------------------------------------------------------------------
    // AuthContext
    // -----------------------------------------------------------------------

    #[test]
    fn auth_context_with_principal() {
        let ctx = AuthContext::new(Some(42_i32));
        assert!(ctx.is_authenticated());
        assert_eq!(ctx.principal(), Some(&42));
    }

    #[test]
    fn auth_context_anonymous() {
        let ctx: AuthContext<i32> = AuthContext::new(None);
        assert!(!ctx.is_authenticated());
        assert_eq!(ctx.principal(), None);
    }

    #[test]
    fn auth_context_clone() {
        let ctx = AuthContext::new(Some("alice"));
        let cloned = ctx.clone();
        assert_eq!(cloned.principal(), Some(&"alice"));
    }

    // -----------------------------------------------------------------------
    // AuthError → HttpError conversion
    // -----------------------------------------------------------------------

    #[test]
    fn unauthorized_converts_to_401() {
        let error: HttpError = AuthError::Unauthorized("bad token".into()).into_http_error();
        assert_eq!(error.status(), HttpStatus::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_converts_to_403() {
        let error: HttpError = AuthError::Forbidden("no access".into()).into_http_error();
        assert_eq!(error.status(), HttpStatus::FORBIDDEN);
    }

    #[test]
    fn configuration_converts_to_500() {
        let error: HttpError = AuthError::Configuration("bad setup".into()).into_http_error();
        assert_eq!(error.status(), HttpStatus::INTERNAL_SERVER_ERROR);
    }

    // -----------------------------------------------------------------------
    // Constructor smoke-tests
    // -----------------------------------------------------------------------

    #[test]
    fn authentication_middleware_constructs() {
        struct DummyAuth;
        impl Authenticator<&'static str> for DummyAuth {
            fn authenticate<'a>(
                &'a self,
                _request: &'a Request,
            ) -> AuthenticationFuture<'a, &'static str> {
                Box::pin(async move { Ok(Some("user")) })
            }
        }
        let _mw: AuthenticationMiddleware<DummyAuth, &'static str> =
            AuthenticationMiddleware::new(DummyAuth);
    }

    #[test]
    fn require_authenticated_constructs() {
        let _guard: RequireAuthenticated<String> = RequireAuthenticated::new();
    }

    #[test]
    fn require_access_constructs() {
        let _role = RequireAccess::<String>::role("admin");
        let _perm = RequireAccess::<String>::permission("read");
    }

    #[test]
    fn access_requirement_equality() {
        assert_eq!(
            AccessRequirement::Role("admin".into()),
            AccessRequirement::Role("admin".into())
        );
        assert_ne!(
            AccessRequirement::Role("admin".into()),
            AccessRequirement::Permission("admin".into())
        );
    }
}
