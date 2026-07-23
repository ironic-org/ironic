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

#[cfg(all(feature = "redis", feature = "sessions"))]
use redis::AsyncCommands;

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use ironic::auth::sessions::SessionId;
    ///
    /// let id = SessionId::parse("a1b2c3d4e5f6071829a0b1c2d3e4f50617283940a1b2c3d4e5f6071829a0b1c0").unwrap();
    /// assert_eq!(id.expose().len(), 64);
    /// assert!(SessionId::parse("short").is_none());
    /// assert!(SessionId::parse("not-hex-garbage!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!").is_none());
    /// ```
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
///
/// # Example
///
/// ```rust
/// use ironic::auth::sessions::cookie_value;
///
/// let header = "session=abc123; theme=dark; csrf=tok";
/// assert_eq!(cookie_value(header, "session"), Some("abc123"));
/// assert_eq!(cookie_value(header, "theme"), Some("dark"));
/// assert_eq!(cookie_value(header, "missing"), None);
/// ```
#[must_use]
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').find_map(|part| {
        let (candidate, value) = part.trim().split_once('=')?;
        (candidate == name).then_some(value)
    })
}

/// Builds a secure host-only session cookie.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use ironic::auth::sessions::{SessionId, session_cookie};
///
/// let id = SessionId::generate().unwrap();
/// let cookie = session_cookie("sid", &id, Duration::from_secs(3600), true);
/// assert!(cookie.starts_with("sid="));
/// assert!(cookie.contains("Max-Age=3600"));
/// assert!(cookie.contains("Secure"));
/// ```
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
///
/// # Example
///
/// ```rust
/// use ironic::auth::sessions::expired_session_cookie;
///
/// let cookie = expired_session_cookie("sid", true);
/// assert!(cookie.starts_with("sid="));
/// assert!(cookie.contains("Max-Age=0"));
/// ```
#[must_use]
pub fn expired_session_cookie(name: &str, secure: bool) -> String {
    let secure_attribute = if secure { "; Secure" } else { "" };
    format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure_attribute}")
}

/// Configuration for [`RedisSessionStore`].
///
/// Controls session TTL in seconds.
#[cfg(all(feature = "redis", feature = "sessions"))]
#[derive(Clone, Copy, Debug)]
pub struct RedisSessionConfig {
    /// Session TTL in seconds (default: 86400 / 24 hours).
    pub session_ttl: u64,
}

#[cfg(all(feature = "redis", feature = "sessions"))]
impl Default for RedisSessionConfig {
    fn default() -> Self {
        Self { session_ttl: 86400 }
    }
}

/// A Redis-backed session store for production deployments.
///
/// Serializes sessions as JSON values under the `ironic:session:{id}` key.
/// Uses Redis TTL for automatic expiry based on the configured `session_ttl`.
///
/// # Example
///
/// ```rust,ignore
/// use ironic::auth::sessions::{RedisSessionStore, RedisSessionConfig, SessionStore};
/// use std::time::Duration;
///
/// let conn = redis::Client::open("redis://127.0.0.1/")?
///     .get_tokio_connection_manager()
///     .await?;
/// let store = RedisSessionStore::new(conn)
///     .with_ttl(Duration::from_secs(3600)); // 1 hour TTL
/// ```
#[cfg(all(feature = "redis", feature = "sessions"))]
#[derive(Clone, Debug)]
pub struct RedisSessionStore {
    connection_manager: redis::aio::ConnectionManager,
    session_ttl: Duration,
}

#[cfg(all(feature = "redis", feature = "sessions"))]
impl RedisSessionStore {
    const KEY_PREFIX: &'static str = "ironic:session:";

    /// Creates a store using an existing Redis connection manager.
    ///
    /// Default TTL is 86400 seconds (24 hours).  Use [`with_ttl`](Self::with_ttl)
    /// or [`with_config`](Self::with_config) to customize.
    #[must_use]
    pub fn new(connection_manager: redis::aio::ConnectionManager) -> Self {
        Self {
            connection_manager,
            session_ttl: Duration::from_hours(24),
        }
    }

    /// Creates a store with a [`RedisSessionConfig`].
    #[must_use]
    pub fn with_config(
        connection_manager: redis::aio::ConnectionManager,
        config: RedisSessionConfig,
    ) -> Self {
        Self {
            connection_manager,
            session_ttl: Duration::from_secs(config.session_ttl),
        }
    }

    /// Overrides the default session TTL.
    #[must_use]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.session_ttl = ttl;
        self
    }
}

#[cfg(all(feature = "redis", feature = "sessions"))]
impl SessionStore for RedisSessionStore {
    fn load<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, Option<Session>> {
        let key = format!("{}{}", Self::KEY_PREFIX, id.expose());
        let mut conn = self.connection_manager.clone();
        Box::pin(async move {
            let data: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| SessionError::Store(format!("Redis GET failed: {e}")))?;

            match data {
                Some(json) => {
                    let value: serde_json::Value =
                        serde_json::from_str(&json).map_err(SessionError::from)?;

                    let id_str = value["id"].as_str().ok_or_else(|| {
                        SessionError::Store("Missing session id in stored data".into())
                    })?;
                    let parsed_id = SessionId::parse(id_str).ok_or_else(|| {
                        SessionError::Store("Invalid session id in stored data".into())
                    })?;
                    let expires_at_secs = value["expires_at"].as_u64().ok_or_else(|| {
                        SessionError::Store("Missing expires_at in stored data".into())
                    })?;
                    let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(expires_at_secs);

                    if expires_at <= SystemTime::now() {
                        let _ = conn.del::<_, ()>(&key).await;
                        return Ok(None);
                    }

                    let values: BTreeMap<String, serde_json::Value> =
                        serde_json::from_value(value["values"].clone())
                            .map_err(SessionError::from)?;

                    Ok(Some(Session {
                        id: parsed_id,
                        expires_at,
                        values,
                    }))
                }
                None => Ok(None),
            }
        })
    }

    fn save(&self, session: Session) -> SessionFuture<'_, ()> {
        let key = format!("{}{}", Self::KEY_PREFIX, session.id.expose());
        let value = serde_json::json!({
            "id": session.id.expose(),
            "expires_at": session.expires_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            "values": session.values,
        });
        let json = serde_json::to_string(&value).map_err(SessionError::from);
        let ttl = self.session_ttl.as_secs();
        let mut conn = self.connection_manager.clone();
        Box::pin(async move {
            let json = json?;
            conn.set_ex::<_, _, ()>(&key, json, ttl)
                .await
                .map_err(|e| SessionError::Store(format!("Redis SETEX failed: {e}")))?;
            Ok(())
        })
    }

    fn delete<'a>(&'a self, id: &'a SessionId) -> SessionFuture<'a, ()> {
        let key = format!("{}{}", Self::KEY_PREFIX, id.expose());
        let mut conn = self.connection_manager.clone();
        Box::pin(async move {
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| SessionError::Store(format!("Redis DEL failed: {e}")))?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // -----------------------------------------------------------------------
    // SessionId
    // -----------------------------------------------------------------------

    #[test]
    fn session_id_generates_64_hex_chars() {
        let id = SessionId::generate().unwrap();
        assert_eq!(id.expose().len(), 64);
        assert!(id.expose().bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn session_id_parse_valid() {
        let hex = "a1b2c3d4e5f6071829a0b1c2d3e4f50617283940a1b2c3d4e5f6071829a0b1c0";
        assert_eq!(hex.len(), 64);
        let parsed = SessionId::parse(hex).unwrap();
        assert_eq!(parsed.expose(), hex);
    }

    #[test]
    fn session_id_parse_invalid_length() {
        assert!(SessionId::parse("too-short").is_none());
        assert!(SessionId::parse("also-w-a-a-a-a-a-a-a-a-a-a-a-a-a-a-a-a-a-a-way-too-longgggggggggggggggggggggggggggggggggggggggggggggggg").is_none());
    }

    #[test]
    fn session_id_parse_non_hex() {
        let hex = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        assert_eq!(hex.len(), 64);
        assert!(SessionId::parse(hex).is_none());
    }

    #[test]
    fn session_id_debug_redacts() {
        let id = SessionId::generate().unwrap();
        let debug = format!("{id:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(id.expose()));
    }

    #[test]
    fn session_id_clone_eq_hash() {
        let a = SessionId::generate().unwrap();
        let b = SessionId::parse(a.expose()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.expose(), b.expose());
    }

    // -----------------------------------------------------------------------
    // Session
    // -----------------------------------------------------------------------

    #[test]
    fn session_new_creates_with_id() {
        let session = Session::new(Duration::from_hours(1)).unwrap();
        assert_eq!(session.id.expose().len(), 64);
    }

    #[test]
    fn session_insert_and_get() {
        let mut session = Session::new(Duration::from_hours(1)).unwrap();
        session.insert("user_id", 42_u64).unwrap();
        let value: Option<u64> = session.get("user_id").unwrap();
        assert_eq!(value, Some(42));
    }

    #[test]
    fn session_get_missing_key() {
        let session = Session::new(Duration::from_hours(1)).unwrap();
        let value: Option<String> = session.get("nonexistent").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn session_get_type_mismatch() {
        let mut session = Session::new(Duration::from_hours(1)).unwrap();
        session.insert("key", "hello").unwrap();
        let result: Result<Option<i32>, _> = session.get("key");
        assert!(result.is_err());
    }

    #[test]
    fn session_multiple_values() {
        let mut session = Session::new(Duration::from_hours(1)).unwrap();
        session.insert("name", "Alice").unwrap();
        session.insert("score", 100_i32).unwrap();
        assert_eq!(session.get::<String>("name").unwrap(), Some("Alice".into()));
        assert_eq!(session.get::<i32>("score").unwrap(), Some(100));
    }

    // -----------------------------------------------------------------------
    // cookie_value
    // -----------------------------------------------------------------------

    #[test]
    fn cookie_value_found() {
        let header = "session=abc123; theme=dark";
        assert_eq!(cookie_value(header, "session"), Some("abc123"));
        assert_eq!(cookie_value(header, "theme"), Some("dark"));
    }

    #[test]
    fn cookie_value_missing() {
        assert_eq!(cookie_value("session=abc123", "missing"), None);
    }

    #[test]
    fn cookie_value_empty_header() {
        assert_eq!(cookie_value("", "anything"), None);
    }

    #[test]
    fn cookie_value_trailing_semicolon() {
        let header = "sid=val;";
        assert_eq!(cookie_value(header, "sid"), Some("val"));
    }

    // -----------------------------------------------------------------------
    // session_cookie / expired_session_cookie
    // -----------------------------------------------------------------------

    #[test]
    fn session_cookie_includes_attributes() {
        let id = SessionId::generate().unwrap();
        let cookie = session_cookie("sid", &id, Duration::from_hours(1), true);
        assert!(cookie.starts_with("sid="));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains(&id.expose().to_string()));
    }

    #[test]
    fn session_cookie_insecure() {
        let id = SessionId::generate().unwrap();
        let cookie = session_cookie("sid", &id, Duration::from_mins(1), false);
        assert!(!cookie.contains("Secure"));
        assert!(cookie.contains("Max-Age=60"));
    }

    #[test]
    fn expired_session_cookie_sets_max_age_zero() {
        let cookie = expired_session_cookie("sid", true);
        assert!(cookie.starts_with("sid="));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn expired_session_cookie_insecure() {
        let cookie = expired_session_cookie("sid", false);
        assert!(!cookie.contains("Secure"));
        assert!(cookie.contains("Max-Age=0"));
    }

    // -----------------------------------------------------------------------
    // InMemorySessionStore (async)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn memory_store_save_and_load() {
        let store = InMemorySessionStore::default();
        let session = Session::new(Duration::from_hours(1)).unwrap();
        let id = session.id.clone();
        store.save(session).await.unwrap();
        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, id);
    }

    #[tokio::test]
    async fn memory_store_load_missing() {
        let store = InMemorySessionStore::default();
        let id = SessionId::generate().unwrap();
        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn memory_store_delete() {
        let store = InMemorySessionStore::default();
        let session = Session::new(Duration::from_hours(1)).unwrap();
        let id = session.id.clone();
        store.save(session).await.unwrap();
        store.delete(&id).await.unwrap();
        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn memory_store_load_expired() {
        let store = InMemorySessionStore::default();
        let mut session = Session::new(Duration::from_hours(1)).unwrap();
        // Force expiry in the past
        session.expires_at = SystemTime::UNIX_EPOCH;
        let id = session.id.clone();
        store.save(session).await.unwrap();
        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_none());
    }
}
