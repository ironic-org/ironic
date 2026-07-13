//! Rate limiting middleware for Ironic.
//!
//! Feature flag: `security-rate-limit`.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use ironic_http::{
    FrameworkResponse, HttpStatus, Middleware, MiddlewareNext, PipelineFuture, RequestContext,
};

/// A sliding window rate limit entry.
#[derive(Clone)]
struct WindowEntry {
    timestamp: Instant,
}

/// In-memory sliding window rate limiter.
#[derive(Clone)]
pub struct InMemoryRateLimiter {
    windows: Arc<Mutex<HashMap<String, Vec<WindowEntry>>>>,
    max_requests: u64,
    window_secs: u64,
}

impl InMemoryRateLimiter {
    /// Creates a new rate limiter with the given limits.
    #[must_use]
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    /// Returns `true` if the request is allowed, `false` if rate-limited.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn check(&self, key: &str) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let entries = windows.entry(key.to_owned()).or_default();

        // Remove expired entries
        entries.retain(|e| now.duration_since(e.timestamp).as_secs() < self.window_secs);

        if entries.len() as u64 >= self.max_requests {
            false
        } else {
            entries.push(WindowEntry { timestamp: now });
            true
        }
    }

    /// Returns the number of remaining requests in the current window.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn remaining(&self, key: &str) -> u64 {
        let windows = self.windows.lock().unwrap();
        let now = Instant::now();
        windows
            .get(key)
            .map_or(self.max_requests, |entries| {
                let active = entries
                    .iter()
                    .filter(|e| now.duration_since(e.timestamp).as_secs() < self.window_secs)
                    .count() as u64;
                self.max_requests.saturating_sub(active)
            })
    }
}

/// Rate limiting middleware with configurable backend.
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<InMemoryRateLimiter>,
}

impl RateLimitMiddleware {
    /// Creates a new rate limit middleware.
    #[must_use]
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::new(max_requests, window_secs)),
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
            // Use client IP or similar identifier as rate limit key
            let key = context
                .request()
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_owned();

            if !self.limiter.check(&key) {
                let mut response = FrameworkResponse::error(
                    HttpStatus::TOO_MANY_REQUESTS,
                    "RF_HTTP_RATE_LIMIT_EXCEEDED",
                    "Too many requests, please try again later",
                );
                response.headers_mut().insert(
                    http::header::RETRY_AFTER,
                    http::HeaderValue::from_static("60"),
                );
                response.headers_mut().insert(
                    http::header::HeaderName::from_static("x-ratelimit-remaining"),
                    http::HeaderValue::from_static("0"),
                );
                return Ok(response);
            }

            let remaining = self.limiter.remaining(&key);
            let mut response = next.run(context).await?;
            response.headers_mut().insert(
                http::header::HeaderName::from_static("x-ratelimit-remaining"),
                http::HeaderValue::from_str(&remaining.to_string())
                    .unwrap_or_else(|_| http::HeaderValue::from_static("0")),
            );
            Ok(response)
        })
    }
}
