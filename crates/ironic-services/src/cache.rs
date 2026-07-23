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
pub enum CacheError {
    /// A value could not be encoded or decoded.
    #[error("IRONIC_CACHE_SERIALIZATION: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A backend operation failed.
    #[error("IRONIC_CACHE_BACKEND: {0}")]
    Backend(String),
}

/// Backend-neutral asynchronous byte cache.
///
/// # Errors
///
/// Each method returns [`CacheError`] on backend failure.
pub trait Cache: Send + Sync + 'static {
    /// Loads a non-expired value.
    fn get<'a>(&'a self, key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>>;
    /// Stores a value with optional time-to-live.
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
    /// Removes all entries whose key starts with given prefix.
    fn remove_by_prefix<'a>(&'a self, prefix: &'a str) -> CacheFuture<'a, usize>;
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

    fn remove_by_prefix<'a>(&'a self, prefix: &'a str) -> CacheFuture<'a, usize> {
        Box::pin(async move {
            let mut entries = self.entries.write().await;
            let keys: Vec<String> = entries
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect();
            let count = keys.len();
            for k in keys {
                entries.remove(&k);
            }
            Ok(count)
        })
    }
}

/// A Redis-backed cache implementation.
///
/// Requires both the `cache` and `redis` features and a running Redis instance.
/// The cache holds a connection manager reference but does not perform
/// automatic reconnection — use a connection manager for production use.
#[cfg(all(feature = "cache", feature = "redis"))]
#[derive(Clone)]
pub struct RedisCache {
    client: ::redis::aio::ConnectionManager,
    key_prefix: String,
}

#[cfg(all(feature = "cache", feature = "redis"))]
impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("key_prefix", &self.key_prefix)
            .field("client", &"ConnectionManager { ... }")
            .finish()
    }
}

#[cfg(all(feature = "cache", feature = "redis"))]
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

#[cfg(all(feature = "cache", feature = "redis"))]
impl Cache for RedisCache {
    fn get<'a>(&'a self, key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>> {
        let full_key = self.prefixed(key);
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            conn.get(&full_key)
                .await
                .map_err(|e| CacheError::Backend(e.to_string()))
        })
    }

    fn set<'a>(
        &'a self,
        key: &'a str,
        value: Vec<u8>,
        ttl: Option<std::time::Duration>,
    ) -> CacheFuture<'a, ()> {
        let full_key = self.prefixed(key);
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            if let Some(duration) = ttl {
                conn.set_ex(&full_key, value, duration.as_secs())
                    .await
                    .map_err(|e| CacheError::Backend(e.to_string()))
            } else {
                conn.set(&full_key, value)
                    .await
                    .map_err(|e| CacheError::Backend(e.to_string()))
            }
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> CacheFuture<'a, bool> {
        let full_key = self.prefixed(key);
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let deleted: i32 = conn
                .del(&full_key)
                .await
                .map_err(|e| CacheError::Backend(e.to_string()))?;
            Ok(deleted > 0)
        })
    }

    fn clear(&self) -> CacheFuture<'_, ()> {
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let mut cursor: u64 = 0;
            loop {
                let (next_cursor, keys): (u64, Vec<String>) = ::redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::Backend(e.to_string()))?;
                cursor = next_cursor;
                if !keys.is_empty() {
                    let _: () = conn
                        .del(&keys)
                        .await
                        .map_err(|e| CacheError::Backend(format!("clear del: {e}")))?;
                }
                if cursor == 0 {
                    break;
                }
            }
            Ok(())
        })
    }

    fn remove_by_prefix<'a>(&'a self, prefix: &'a str) -> CacheFuture<'a, usize> {
        let full_prefix = if self.key_prefix.is_empty() {
            format!("{prefix}*")
        } else {
            format!("{}:{}*", self.key_prefix, prefix)
        };
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let mut total = 0usize;
            let mut cursor: u64 = 0;
            loop {
                let (next_cursor, keys): (u64, Vec<String>) = ::redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&full_prefix)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::Backend(e.to_string()))?;
                cursor = next_cursor;
                let count = keys.len();
                if count > 0 {
                    let _: () = conn
                        .del(&keys)
                        .await
                        .map_err(|e| CacheError::Backend(format!("prefix del: {e}")))?;
                    total += count;
                }
                if cursor == 0 {
                    break;
                }
            }
            Ok(total)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn set_and_get() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache.set("key1", b"value1".to_vec(), None).await.unwrap();
        assert_eq!(cache.get("key1").await.unwrap(), Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn get_missing() {
        let cache = Arc::new(InMemoryCache::default());
        assert_eq!(cache.get("missing").await.unwrap(), None);
    }

    #[tokio::test]
    async fn remove() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache.set("k", vec![1], None).await.unwrap();
        assert!(cache.remove("k").await.unwrap());
        assert!(!cache.remove("k").await.unwrap());
        assert_eq!(cache.get("k").await.unwrap(), None);
    }

    #[tokio::test]
    async fn ttl_expiry() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache
            .set("k", vec![1], Some(Duration::from_millis(1)))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(cache.get("k").await.unwrap(), None);
    }

    #[tokio::test]
    async fn clear() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache.set("a", vec![1], None).await.unwrap();
        cache.set("b", vec![2], None).await.unwrap();
        cache.clear().await.unwrap();
        assert!(cache.get("a").await.unwrap().is_none());
        assert!(cache.get("b").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn remove_by_prefix() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache.set("aa:1", vec![1], None).await.unwrap();
        cache.set("aa:2", vec![2], None).await.unwrap();
        cache.set("bb:1", vec![3], None).await.unwrap();
        assert_eq!(cache.remove_by_prefix("aa:").await.unwrap(), 2);
        assert!(cache.get("aa:1").await.unwrap().is_none());
        assert_eq!(cache.get("bb:1").await.unwrap(), Some(vec![3]));
    }

    #[tokio::test]
    async fn evict_when_at_capacity() {
        let cache = Arc::new(InMemoryCache::new(2));
        cache.set("a", vec![1], None).await.unwrap();
        cache.set("b", vec![2], None).await.unwrap();
        cache.set("c", vec![3], None).await.unwrap();
        assert_eq!(cache.get("c").await.unwrap(), Some(vec![3]));
        assert!(cache.get("a").await.unwrap().is_none() || cache.get("b").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn set_json_and_get_json() {
        let cache = Arc::new(InMemoryCache::new(16));
        let vals = vec!["hello", "world"];
        cache.set_json("k", &vals, None).await.unwrap();
        let got: Option<Vec<String>> = cache.get_json("k").await.unwrap();
        assert_eq!(got, Some(vec!["hello".to_string(), "world".to_string()]));
    }

    #[tokio::test]
    async fn get_json_invalid() {
        let cache = Arc::new(InMemoryCache::new(16));
        cache.set("k", b"not json".to_vec(), None).await.unwrap();
        assert!(cache.get_json::<Vec<String>>("k").await.is_err());
    }

    #[tokio::test]
    async fn insert_many_with_default_capacity() {
        let cache = Arc::new(InMemoryCache::default());
        for i in 0..1000 {
            let key = format!("k{i}");
            cache.set(&key, vec![], None).await.unwrap();
        }
        let count = cache.remove_by_prefix("k").await.unwrap();
        assert_eq!(count, 1000);
    }
}
