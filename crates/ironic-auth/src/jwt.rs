//! Signed JSON Web Token helpers and bearer authentication.

use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};

use super::{AuthError, AuthenticationFuture, Authenticator, bearer_token};
use crate::Request;

/// The upstream `jsonwebtoken` API.
pub use ::jsonwebtoken as driver;

/// Reusable JWT signing and validation keys with an explicit algorithm policy.
#[derive(Clone)]
pub struct JwtService {
    header: driver::Header,
    encoding_key: driver::EncodingKey,
    decoding_key: driver::DecodingKey,
    validation: driver::Validation,
}

impl JwtService {
    /// Creates a symmetric JWT service for an HMAC algorithm.
    #[must_use]
    pub fn hmac(secret: &[u8], algorithm: driver::Algorithm) -> Self {
        let header = driver::Header::new(algorithm);
        let validation = driver::Validation::new(algorithm);
        Self {
            header,
            encoding_key: driver::EncodingKey::from_secret(secret),
            decoding_key: driver::DecodingKey::from_secret(secret),
            validation,
        }
    }

    /// Creates a service from reusable native keys and policies.
    #[must_use]
    pub const fn new(
        header: driver::Header,
        encoding_key: driver::EncodingKey,
        decoding_key: driver::DecodingKey,
        validation: driver::Validation,
    ) -> Self {
        Self {
            header,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Returns mutable validation policy for issuer, audience, leeway, and required claims.
    #[must_use]
    pub const fn validation_mut(&mut self) -> &mut driver::Validation {
        &mut self.validation
    }

    /// Signs claims.
    ///
    /// # Errors
    ///
    /// Returns the upstream JWT error if claims cannot be serialized or signed.
    pub fn encode<C: Serialize>(&self, claims: &C) -> Result<String, driver::errors::Error> {
        driver::encode(&self.header, claims, &self.encoding_key)
    }

    /// Verifies a token signature and validation policy, then returns its claims.
    ///
    /// # Errors
    ///
    /// Returns the upstream JWT error when verification or claim validation fails.
    pub fn decode<C: DeserializeOwned>(
        &self,
        token: &str,
    ) -> Result<driver::TokenData<C>, driver::errors::Error> {
        driver::decode(token, &self.decoding_key, &self.validation)
    }
}

/// Authenticates bearer JWT claims and maps them into an application principal.
pub struct JwtBearerAuthenticator<C, P, F> {
    service: JwtService,
    map_claims: F,
    marker: PhantomData<fn(C) -> P>,
}

impl<C, P, F> JwtBearerAuthenticator<C, P, F> {
    /// Creates an authenticator from a JWT service and claims mapper.
    #[must_use]
    pub const fn new(service: JwtService, map_claims: F) -> Self {
        Self {
            service,
            map_claims,
            marker: PhantomData,
        }
    }
}

impl<C, P, F> Authenticator<P> for JwtBearerAuthenticator<C, P, F>
where
    C: DeserializeOwned + Send + Sync + 'static,
    P: Send + Sync + 'static,
    F: Fn(C) -> Result<P, AuthError> + Send + Sync + 'static,
{
    fn authenticate<'a>(&'a self, request: &'a Request) -> AuthenticationFuture<'a, P> {
        Box::pin(async move {
            let Some(token) = bearer_token(request)? else {
                return Ok(None);
            };
            let claims = self
                .service
                .decode::<C>(token)
                .map_err(|_| AuthError::Unauthorized("Bearer token is invalid or expired".into()))?
                .claims;
            (self.map_claims)(claims).map(Some)
        })
    }
}
