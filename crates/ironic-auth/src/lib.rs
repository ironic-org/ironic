//! Authentication, password hashing, and authorization pipeline helpers.

use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    Request, Guard, GuardDecision, GuardFuture, HttpError, Middleware, MiddlewareNext,
    PipelineFuture, RequestContext,
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
/// # Errors
///
/// Returns [`AuthError::Unauthorized`] when an authorization header is malformed.
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
