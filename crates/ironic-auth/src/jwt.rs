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
    ///
    /// # Example
    ///
    /// ```rust
    /// use ironic::auth::jwt::JwtService;
    /// use jsonwebtoken::Algorithm;
    ///
    /// let service = JwtService::hmac(b"my-secret-key", Algorithm::HS256);
    /// let payload = serde_json::json!({"sub": "hello world", "exp": 9999999999_u64});
    /// let token = service.encode(&payload).unwrap();
    /// let claims: serde_json::Value = service.decode(&token).unwrap().claims;
    /// assert_eq!(claims["sub"], "hello world");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use ironic::auth::jwt::JwtService;
    /// use jsonwebtoken::{Algorithm, Header, EncodingKey, DecodingKey, Validation};
    ///
    /// let header = Header::new(Algorithm::HS384);
    /// let encoding_key = EncodingKey::from_secret(b"key");
    /// let decoding_key = DecodingKey::from_secret(b"key");
    /// let validation = Validation::new(Algorithm::HS384);
    /// let service = JwtService::new(header, encoding_key, decoding_key, validation);
    /// let payload = serde_json::json!({"data": "value", "exp": 9999999999_u64});
    /// assert!(service.encode(&payload).is_ok());
    /// ```
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
    ///
    /// The `map_claims` closure receives decoded claims and should return the
    /// application principal or an [`AuthError`].
    ///
    /// [`AuthError`]: super::AuthError
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

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Algorithm;

    #[test]
    fn hmac_round_trip() {
        let service = JwtService::hmac(b"test-key-12345678", Algorithm::HS256);
        let payload = serde_json::json!({"sub": "hello world", "exp": 9999999999_u64});
        let token = service.encode(&payload).unwrap();
        let claims: serde_json::Value = service.decode(&token).unwrap().claims;
        assert_eq!(claims["sub"], "hello world");
    }

    #[test]
    fn decode_with_wrong_key_fails() {
        let good = JwtService::hmac(b"correct-key-12345678", Algorithm::HS256);
        let evil = JwtService::hmac(b"evil-key-12345678", Algorithm::HS256);
        let payload = "secret data".to_string();
        let token = good.encode(&payload).unwrap();
        assert!(evil.decode::<String>(&token).is_err());
    }

    #[test]
    fn decode_tampered_token_fails() {
        let service = JwtService::hmac(b"my-secret", Algorithm::HS256);
        let payload = "payload".to_string();
        let token = service.encode(&payload).unwrap();
        let mut bytes: Vec<u8> = token.into_bytes();
        if let Some(b) = bytes.last_mut() {
            *b ^= 0x01;
        }
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(service.decode::<String>(&tampered).is_err());
    }

    #[test]
    fn decode_garbage_token_fails() {
        let service = JwtService::hmac(b"secret", Algorithm::HS256);
        assert!(service.decode::<String>("not-a-jwt").is_err());
    }

    #[test]
    fn encode_decode_with_different_algorithms() {
        let service = JwtService::hmac(b"key", Algorithm::HS512);
        let payload = serde_json::json!({"value": 42, "exp": 9999999999_u64});
        let token = service.encode(&payload).unwrap();
        let claims: serde_json::Value = service.decode(&token).unwrap().claims;
        assert_eq!(claims["value"], 42);
    }

    #[test]
    fn jwt_bearer_authenticator_constructs() {
        use super::super::AuthError;
        let service = JwtService::hmac(b"key", Algorithm::HS256);
        let authenticator = JwtBearerAuthenticator::<String, String, _>::new(
            service,
            |claims: String| -> Result<String, AuthError> { Ok(claims) },
        );
        let _ = authenticator;
    }
}
