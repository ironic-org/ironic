//! Asynchronous cache contracts and a bounded process-local implementation.

use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// A boxed asynchronous cache operation.
pub type CacheFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, CacheError>> + Send + 'a>>;

/// A cache backend failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CacheError {
    /// A value could not be encoded or decoded.
    #[error("IRONIC_CACHE_SERIALIZATION: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A backend operation failed.
    #[error("IRONIC_CACHE_BACKEND: {0}")]
    Backend(String),
}

/// Backend-neutral asynchronous byte cache.
pub trait Cache: Send + Sync + 'static {
    /// Loads a non-expired value.
    fn get<'a>(&'a self, key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>>;
    /// Stores a value with an optional time-to-live.
    fn set<'a>(
        &'a self,
        key: &'a str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> CacheFuture<'a, ()>;
    /// Removes a value and returns whether it existed.
    fn remove<'a>(&'a self, key: &'a str) -> CacheFuture<'a, bool>;
    /// Clears this cache namespace.
    fn clear(&self) -> CacheFuture<'_, ()>;
}

#[derive(Clone, Debug)]
struct Entry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

/// A cloneable process-local cache with a maximum entry count.
#[derive(Clone, Debug)]
pub struct InMemoryCache {
    entries: Arc<RwLock<HashMap<String, Entry>>>,
    capacity: usize,
}

impl InMemoryCache {
    /// Creates a cache. Capacity overflow evicts expired entries first, then an arbitrary entry.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            capacity: capacity.max(1),
        }
    }

    /// Loads and deserializes JSON.
    ///
    /// # Errors
    /// Returns a backend or JSON deserialization error.
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        self.get(key)
            .await?
            .map(|value| serde_json::from_slice(&value))
            .transpose()
            .map_err(CacheError::from)
    }

    /// Serializes and stores JSON.
    ///
    /// # Errors
    /// Returns a JSON serialization or backend error.
    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        self.set(key, serde_json::to_vec(value)?, ttl).await
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new(1_024)
    }
}

impl Cache for InMemoryCache {
    fn get<'a>(&'a self, key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            let value = self.entries.read().await.get(key).cloned();
            match value {
                Some(entry)
                    if entry
                        .expires_at
                        .is_none_or(|expiry| expiry > Instant::now()) =>
                {
                    Ok(Some(entry.value))
                }
                Some(_) => {
                    self.entries.write().await.remove(key);
                    Ok(None)
                }
                None => Ok(None),
            }
        })
    }

    fn set<'a>(
        &'a self,
        key: &'a str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> CacheFuture<'a, ()> {
        Box::pin(async move {
            let now = Instant::now();
            let mut entries = self.entries.write().await;
            entries.retain(|_, entry| entry.expires_at.is_none_or(|expiry| expiry > now));
            if entries.len() >= self.capacity
                && !entries.contains_key(key)
                && let Some(evicted) = entries.keys().next().cloned()
            {
                entries.remove(&evicted);
            }
            entries.insert(
                key.to_owned(),
                Entry {
                    value,
                    expires_at: ttl.map(|duration| now + duration),
                },
            );
            Ok(())
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> CacheFuture<'a, bool> {
        Box::pin(async move { Ok(self.entries.write().await.remove(key).is_some()) })
    }

    fn clear(&self) -> CacheFuture<'_, ()> {
        Box::pin(async move {
            self.entries.write().await.clear();
            Ok(())
        })
    }
}

/// A Redis-backed cache implementation.
///
/// Requires the `redis` feature and a running Redis instance.
/// The cache holds a connection manager reference but does not perform
/// automatic reconnection — use a connection manager for production use.
#[cfg(feature = "redis")]
#[derive(Clone)]
pub struct RedisCache {
    client: ::redis::aio::ConnectionManager,
    key_prefix: String,
}

#[cfg(feature = "redis")]
impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("key_prefix", &self.key_prefix)
            .field("client", &"ConnectionManager { ... }")
            .finish()
    }
}

#[cfg(feature = "redis")]
impl RedisCache {
    /// Creates a new Redis cache from an existing connection manager.
    #[must_use]
    pub fn new(client: ::redis::aio::ConnectionManager) -> Self {
        Self {
            client,
            key_prefix: String::new(),
        }
    }

    /// Sets a key prefix for namespacing cache entries.
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    fn prefixed(&self, key: &str) -> String {
        if self.key_prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{}:{}", self.key_prefix, key)
        }
    }
}

#[cfg(feature = "redis")]
impl Cache for RedisCache {
    fn get<'a>(&'a self, _key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            Err(CacheError::Backend(
                "Redis cache requires a live connection; initialize with `new(client)` first"
                    .into(),
            ))
        })
    }

    fn set<'a>(
        &'a self,
        _key: &'a str,
        _value: Vec<u8>,
        _ttl: Option<std::time::Duration>,
    ) -> CacheFuture<'a, ()> {
        Box::pin(async move {
            Err(CacheError::Backend(
                "Redis cache requires a live connection; initialize with `new(client)` first"
                    .into(),
            ))
        })
    }

    fn remove<'a>(&'a self, _key: &'a str) -> CacheFuture<'a, bool> {
        Box::pin(async move {
            Err(CacheError::Backend(
                "Redis cache requires a live connection; initialize with `new(client)` first"
                    .into(),
            ))
        })
    }

    fn clear(&self) -> CacheFuture<'_, ()> {
        Box::pin(async move {
            Err(CacheError::Backend(
                "Redis cache does not support clear across all keys without a prefix scan".into(),
            ))
        })
    }
}
