//! Request session storage and secure identifier generation.

use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;

/// A boxed asynchronous session-store operation.
pub type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, SessionError>> + Send + 'a>>;

/// An opaque, cryptographically random session identifier.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SessionId(String);

impl SessionId {
    /// Generates a 256-bit identifier from the operating system random source.
    ///
    /// # Errors
    ///
    /// Returns an error if the operating system random source is unavailable.
    pub fn generate() -> Result<Self, SessionError> {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|error| SessionError::Random(error.to_string()))?;
        let mut encoded = String::with_capacity(64);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Ok(Self(encoded))
    }

    /// Parses a session identifier, rejecting unexpected lengths and characters.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| Self(value.to_owned()))
    }

    /// Returns the identifier for cookie or store lookup use.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SessionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SessionId([REDACTED])")
    }
}

/// Serializable data and expiry associated with a session.
#[derive(Clone, Debug)]
pub struct Session {
    /// Opaque session identifier.
    pub id: SessionId,
    /// Absolute expiry time.
    pub expires_at: SystemTime,
    values: BTreeMap<String, serde_json::Value>,
}

impl Session {
    /// Creates an empty session with a generated identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if secure identifier generation fails.
    pub fn new(ttl: Duration) -> Result<Self, SessionError> {
        Ok(Self {
            id: SessionId::generate()?,
            expires_at: SystemTime::now() + ttl,
            values: BTreeMap::new(),
        })
    }

    /// Inserts a typed JSON value.
    ///
    /// # Errors
    ///
    /// Returns a serialization error when `value` cannot be represented as JSON.
    pub fn insert<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), SessionError> {
        self.values.insert(key.into(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Deserializes a typed value from the session.
    ///
    /// # Errors
    ///
    /// Returns a deserialization error when the stored JSON does not match `T`.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SessionError> {
        self.values
            .get(key)
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(SessionError::from)
    }
}

/// A session persistence failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SessionError {
    /// Secure random generation failed.
    #[error("IRONIC_SESSION_RANDOM: {0}")]
    Random(String),
    /// Session data could not be encoded or decoded.
    #[error("IRONIC_SESSION_SERIALIZATION: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A persistence backend failed.
    #[error("IRONIC_SESSION_STORE: {0}")]
    Store(String),
}

/// An asynchronous persistence boundary for application sessions.
pub trait SessionStore: Send + Sync + 'static {
    /// Loads an unexpired session.
    fn load<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, Option<Session>>;

    /// Creates or replaces a session.
    fn save(&self, session: Session) -> SessionFuture<'_, ()>;

    /// Deletes a session.
    fn delete<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, ()>;
}

/// A process-local session store for development and single-process applications.
#[derive(Clone, Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl SessionStore for InMemorySessionStore {
    fn load<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, Option<Session>> {
        Box::pin(async move {
            let session = self.sessions.read().await.get(id).cloned();
            if session
                .as_ref()
                .is_some_and(|value| value.expires_at > SystemTime::now())
            {
                Ok(session)
            } else {
                self.sessions.write().await.remove(id);
                Ok(None)
            }
        })
    }

    fn save(&self, session: Session) -> SessionFuture<'_, ()> {
        Box::pin(async move {
            self.sessions
                .write()
                .await
                .insert(session.id.clone(), session);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, ()> {
        Box::pin(async move {
            self.sessions.write().await.remove(id);
            Ok(())
        })
    }
}

/// Finds a named cookie in an HTTP `Cookie` header value.
#[must_use]
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').find_map(|part| {
        let (candidate, value) = part.trim().split_once('=')?;
        (candidate == name).then_some(value)
    })
}

/// Builds a secure host-only session cookie.
#[must_use]
pub fn session_cookie(name: &str, id: &SessionId, max_age: Duration, secure: bool) -> String {
    let secure_attribute = if secure { "; Secure" } else { "" };
    format!(
        "{name}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        id.expose(),
        max_age.as_secs(),
        secure_attribute
    )
}

/// Builds a cookie that immediately removes a browser session identifier.
#[must_use]
pub fn expired_session_cookie(name: &str, secure: bool) -> String {
    let secure_attribute = if secure { "; Secure" } else { "" };
    format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_attribute}")
}
