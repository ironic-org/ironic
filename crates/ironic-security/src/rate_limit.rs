//! Rate limiting middleware for Ironic.
//!
//! Feature flag: `security-rate-limit`.
//!
//! Provides a pluggable [`RateLimitBackend`] trait so that in-memory and
//! Redis-backed limiters can be used interchangeably.

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ironic_http::{
    FrameworkResponse, HttpStatus, Middleware, MiddlewareNext, PipelineFuture, RequestContext,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// The result of a rate-limit check.
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed through.
    pub allowed: bool,
    /// How many requests the client may still send within the window.
    pub remaining: u64,
    /// Best-effort estimate of seconds until the window resets.
    pub reset_after: Duration,
}

/// Pluggable rate-limit backend.
///
/// # Lifetime
///
/// Because the returned future borrows `self`, the return type uses
/// `Pin<Box<dyn Future + Send>>` rather than `async fn` so the trait
/// remains object-safe for `dyn RateLimitBackend`.
pub trait RateLimitBackend: Send + Sync {
    /// Check whether `key` has exceeded `max_requests` within `window_secs`.
    fn check_rate_limit<'a>(
        &'a self,
        key: &'a str,
        max_requests: u64,
        window_secs: u64,
    ) -> Pin<Box<dyn Future<Output = RateLimitResult> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// In-memory backend
// ---------------------------------------------------------------------------

/// In-memory sliding-window rate limiter.
#[derive(Clone)]
pub struct InMemoryRateLimiter {
    windows: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl InMemoryRateLimiter {
    /// Creates an empty rate limiter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns `true` if the request is allowed, `false` if rate-limited.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn check(&self, key: &str, max_requests: u64, window_secs: u64) -> bool {
        let result = self.sync_check(key, max_requests, window_secs);
        result.allowed
    }

    /// Returns the number of remaining requests in the current window
    /// without recording a new request.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn remaining(&self, key: &str, max_requests: u64, window_secs: u64) -> u64 {
        let windows = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        windows.get(key).map_or(max_requests, |entries| {
            let active = entries
                .iter()
                .filter(|e| now.duration_since(**e).as_secs() < window_secs)
                .count() as u64;
            max_requests.saturating_sub(active)
        })
    }

    /// Shared synchronous implementation used by both `check` and `remaining`.
    fn sync_check(&self, key: &str, max_requests: u64, window_secs: u64) -> RateLimitResult {
        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        let entries = windows.entry(key.to_owned()).or_default();

        entries.retain(|e| now.duration_since(*e).as_secs() < window_secs);

        let oldest = entries.first().copied();
        let active = entries.len() as u64;

        if active >= max_requests {
            let reset_after = oldest.map_or(Duration::from_secs(window_secs), |t| {
                let elapsed = now.duration_since(t).as_secs();
                Duration::from_secs(window_secs.saturating_sub(elapsed))
            });
            RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_after,
            }
        } else {
            entries.push(now);
            let reset_after = Duration::from_secs(window_secs);
            RateLimitResult {
                allowed: true,
                remaining: max_requests.saturating_sub(active + 1),
                reset_after,
            }
        }
    }
}

impl Default for InMemoryRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitBackend for InMemoryRateLimiter {
    fn check_rate_limit<'a>(
        &'a self,
        key: &'a str,
        max_requests: u64,
        window_secs: u64,
    ) -> Pin<Box<dyn Future<Output = RateLimitResult> + Send + 'a>> {
        let result = self.sync_check(key, max_requests, window_secs);
        Box::pin(async move { result })
    }
}

// ---------------------------------------------------------------------------
// Redis backend (behind the `redis` feature flag)
// ---------------------------------------------------------------------------

#[cfg(feature = "redis")]
/// Redis-backed sliding-window rate limiter.
///
/// Uses INCR + EXPIRE with a compound key `ratelimit:{key}:{window_secs}`.
/// The window is reset by Redis's automatic key expiry.
pub struct RedisRateLimiter {
    connection: ::redis::aio::ConnectionManager,
}

#[cfg(feature = "redis")]
impl RedisRateLimiter {
    /// Creates a rate limiter that uses an existing Redis connection manager.
    #[must_use]
    pub fn new(connection: ::redis::aio::ConnectionManager) -> Self {
        Self { connection }
    }
}

#[cfg(feature = "redis")]
impl RateLimitBackend for RedisRateLimiter {
    fn check_rate_limit<'a>(
        &'a self,
        key: &'a str,
        max_requests: u64,
        window_secs: u64,
    ) -> Pin<Box<dyn Future<Output = RateLimitResult> + Send + 'a>> {
        Box::pin(async move {
            let redis_key = format!("ratelimit:{key}:{window_secs}");
            let mut conn = self.connection.clone();
            let result: Result<u64, _> = ::redis::pipe()
                .atomic()
                .cmd("INCR")
                .arg(&redis_key)
                .ignore()
                .cmd("EXPIRE")
                .arg(&redis_key)
                .arg(window_secs)
                .ignore()
                .cmd("GET")
                .arg(&redis_key)
                .query_async(&mut conn)
                .await;
            match result {
                Ok(current) => {
                    if current > max_requests {
                        let ttl: Result<u64, _> = ::redis::cmd("TTL")
                            .arg(&redis_key)
                            .query_async(&mut conn)
                            .await;
                        let reset_after = Duration::from_secs(ttl.unwrap_or(window_secs));
                        RateLimitResult {
                            allowed: false,
                            remaining: 0,
                            reset_after,
                        }
                    } else {
                        RateLimitResult {
                            allowed: true,
                            remaining: max_requests.saturating_sub(current),
                            reset_after: Duration::from_secs(window_secs),
                        }
                    }
                }
                Err(_) => {
                    // Redis unavailable — allow the request through
                    // so the application stays up.
                    RateLimitResult {
                        allowed: true,
                        remaining: max_requests,
                        reset_after: Duration::from_secs(window_secs),
                    }
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Rate-limiting middleware with a pluggable backend.
#[derive(Clone)]
pub struct RateLimitMiddleware {
    backend: Arc<dyn RateLimitBackend>,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimitMiddleware {
    /// Creates a new middleware backed by an in-memory rate limiter.
    #[must_use]
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            backend: Arc::new(InMemoryRateLimiter::new()),
            max_requests,
            window_secs,
        }
    }

    /// Creates a new middleware with a custom backend (e.g. Redis).
    #[must_use]
    pub fn with_backend(
        backend: Arc<dyn RateLimitBackend>,
        max_requests: u64,
        window_secs: u64,
    ) -> Self {
        Self {
            backend,
            max_requests,
            window_secs,
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let key = context
                .request()
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|all| all.split(',').next_back().map(str::trim))
                .filter(|ip| !ip.is_empty())
                .unwrap_or("127.0.0.1")
                .to_owned();

            let result = self
                .backend
                .check_rate_limit(&key, self.max_requests, self.window_secs)
                .await;

            if !result.allowed {
                let mut response = FrameworkResponse::error(
                    HttpStatus::TOO_MANY_REQUESTS,
                    ironic_core::error_codes::codes::RATE_LIMIT_EXCEEDED,
                    "Too many requests, please try again later",
                );
                response.headers_mut().insert(
                    http::header::RETRY_AFTER,
                    http::HeaderValue::from_str(&result.reset_after.as_secs().to_string())
                        .unwrap_or_else(|_| http::HeaderValue::from_static("60")),
                );
                response.headers_mut().insert(
                    http::header::HeaderName::from_static("x-ratelimit-remaining"),
                    http::HeaderValue::from_static("0"),
                );
                response.headers_mut().insert(
                    http::header::HeaderName::from_static("x-ratelimit-limit"),
                    http::HeaderValue::from_str(&self.max_requests.to_string())
                        .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
                );
                response.headers_mut().insert(
                    http::header::HeaderName::from_static("x-ratelimit-reset"),
                    http::HeaderValue::from_str(&result.reset_after.as_secs().to_string())
                        .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
                );
                return Ok(response);
            }

            let mut response = next.run(context).await?;
            response.headers_mut().insert(
                http::header::HeaderName::from_static("x-ratelimit-remaining"),
                http::HeaderValue::from_str(&result.remaining.to_string())
                    .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
            );
            response.headers_mut().insert(
                http::header::HeaderName::from_static("x-ratelimit-limit"),
                http::HeaderValue::from_str(&self.max_requests.to_string())
                    .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
            );
            response.headers_mut().insert(
                http::header::HeaderName::from_static("x-ratelimit-reset"),
                http::HeaderValue::from_str(&result.reset_after.as_secs().to_string())
                    .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
            );
            Ok(response)
        })
    }
}
