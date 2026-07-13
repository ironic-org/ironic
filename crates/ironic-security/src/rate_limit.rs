//! Rate limiting middleware for Ironic.
//!
//! Provides sliding-window rate limiting with configurable limits and
//! an in-memory backend for development. Production deployments should
//! use the Redis backend.
//!
//! Feature flag: `security-rate-limit`.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

use ironic_http::{HttpError, HttpStatus, Middleware, MiddlewareNext, PipelineFuture, RequestContext};

/// A rate-limit configuration entry.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window.
    pub max_requests: u64,
    /// Duration of the sliding window.
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_mins(1),
        }
    }
}

struct WindowEntry {
    count: u64,
    window_start: Instant,
}

/// Sliding-window rate limiter backed by an in-memory store.
///
/// For production, implement the [`RateLimitBackend`] trait with Redis.
pub struct InMemoryRateLimiter {
    config: RateLimitConfig,
    clients: Mutex<HashMap<SocketAddr, WindowEntry>>,
}

impl InMemoryRateLimiter {
    /// Creates a new rate limiter with the given configuration.
    #[must_use]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Mutex::new(HashMap::new()),
        }
    }

    fn check(&self, addr: SocketAddr) -> Result<(), HttpError> {
        let now = Instant::now();
        let mut clients = self.clients.lock().expect("rate limit lock poisoned");

        let entry = clients.entry(addr).or_insert(WindowEntry {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) > self.config.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        if entry.count > self.config.max_requests {
            return Err(HttpError::new(
                HttpStatus::TOO_MANY_REQUESTS,
                "RF_RATE_LIMITED",
                "Too many requests. Please try again later.",
            ));
        }

        Ok(())
    }
}

/// Abstract rate-limit backend for production deployments.
#[cfg(feature = "redis")]
pub trait RateLimitBackend: Send + Sync + 'static {
    /// Checks and increments the rate-limit counter for the given key.
    fn check_and_increment(&self, key: &str, max_requests: u64, window_secs: u64) -> bool;
}

impl Middleware for InMemoryRateLimiter {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        let addr = context
            .extension::<SocketAddr>()
            .copied()
            .unwrap_or_else(|| ([0, 0, 0, 0], 0).into());

        if let Err(error) = self.check(addr) {
            return Box::pin(async move { Err(error) });
        }

        Box::pin(async move { next.run(context).await })
    }
}
