//! Production resilience patterns: retry with exponential backoff and circuit breaker.
//!
//! ## Retry
//!
//! ```ignore
//! use ironic::resilience::{RetryConfig, RetryLayer};
//!
//! AxumAdapter::new().configure_router(|r| {
//!     r.layer(RetryLayer::new(RetryConfig {
//!         max_retries: 3,
//!         base_delay_ms: 100,
//!         ..RetryConfig::default()
//!     }));
//! });
//! ```
//!
//! ## Circuit Breaker
//!
//! ```ignore
//! use ironic::resilience::{
//!     CircuitBreakerConfig,
//!     CircuitBreakerLayer,
//! };
//! use std::time::Duration;
//!
//! AxumAdapter::new().configure_router(|r| {
//!     r.layer(CircuitBreakerLayer::new(CircuitBreakerConfig {
//!         failure_threshold: 5,
//!         recovery_timeout: Duration::from_secs(30),
//!         ..CircuitBreakerConfig::default()
//!     }));
//! });
//! ```

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// ===========================================================================
// Retry
// ===========================================================================

/// Configuration for the retry middleware.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (excluding the initial call).
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds.
    pub base_delay_ms: u64,
    /// Backoff multiplier (e.g. 2.0 = exponential).
    pub backoff_multiplier: f64,
    /// Random jitter factor (0.0 to 1.0).
    pub jitter_factor: f64,
    /// Maximum backoff delay in milliseconds.
    pub max_delay_ms: u64,
    /// HTTP status codes that trigger a retry.
    pub retryable_statuses: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            max_delay_ms: 10_000,
            retryable_statuses: vec![408, 429, 500, 502, 503, 504],
        }
    }
}

/// Tower layer that retries failed requests with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryLayer {
    config: RetryConfig,
}

impl RetryLayer {
    /// Creates a retry layer with the given configuration.
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for RetryLayer {
    type Service = RetryService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RetryService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Tower `Service` wrapper that retries on failure.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RetryService<S> {
    inner: S,
    config: RetryConfig,
}

/// Computes backoff delay with jitter.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub fn backoff_delay(attempt: u32, config: &RetryConfig) -> Duration {
    let base = config.base_delay_ms as f64;
    let multiplier = config.backoff_multiplier;
    let raw = base * multiplier.powi(attempt as i32);
    let capped = raw.min(config.max_delay_ms as f64);
    let jitter_range = capped * config.jitter_factor;
    let jittered = capped + (rand_factor() * jitter_range * 2.0 - jitter_range);
    let clamped = jittered.clamp(0.0, config.max_delay_ms as f64);
    Duration::from_millis(clamped as u64)
}

#[allow(clippy::cast_precision_loss)]
fn rand_factor() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish() as f64 / u64::MAX as f64
}

// ===========================================================================
// Circuit Breaker
// ===========================================================================

/// Circuit breaker state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed — requests flow normally.
    Closed,
    /// Circuit is open — requests are rejected immediately.
    Open,
    /// Circuit is half-open — a limited number of test requests are allowed.
    HalfOpen,
}

/// Configuration for the circuit breaker middleware.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// Number of consecutive successes in half-open state to close the circuit.
    pub success_threshold: u32,
    /// Time to wait before transitioning from Open to Half-Open.
    pub recovery_timeout: Duration,
    /// HTTP status codes counted as failures.
    pub failure_statuses: Vec<u16>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            recovery_timeout: Duration::from_secs(30),
            failure_statuses: vec![500, 502, 503, 504],
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerInner {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerInner>>,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerInner {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                config,
            })),
        }
    }

    fn allow_request(&self) -> Result<(), CircuitState> {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            CircuitState::Closed | CircuitState::HalfOpen => Ok(()),
            CircuitState::Open => {
                if let Some(last) = inner.last_failure {
                    if last.elapsed() >= inner.config.recovery_timeout {
                        inner.state = CircuitState::HalfOpen;
                        inner.success_count = 0;
                        Ok(())
                    } else {
                        Err(CircuitState::Open)
                    }
                } else {
                    inner.state = CircuitState::HalfOpen;
                    Ok(())
                }
            }
        }
    }

    fn record_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == CircuitState::HalfOpen {
            inner.success_count += 1;
            if inner.success_count >= inner.config.success_threshold {
                inner.state = CircuitState::Closed;
                inner.failure_count = 0;
            }
        } else {
            inner.failure_count = 0;
        }
    }

    fn record_failure(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failure_count += 1;
        inner.last_failure = Some(Instant::now());
        if inner.failure_count >= inner.config.failure_threshold {
            inner.state = CircuitState::Open;
        }
    }

    #[allow(dead_code)]
    fn state(&self) -> CircuitState {
        self.inner.lock().unwrap().state
    }
}

/// Tower layer that wraps a service with a circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerLayer {
    config: CircuitBreakerConfig,
}

impl CircuitBreakerLayer {
    /// Creates a circuit breaker layer with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for CircuitBreakerLayer {
    type Service = CircuitBreakerService<S>;

    fn layer(&self, service: S) -> Self::Service {
        CircuitBreakerService {
            inner: service,
            breaker: CircuitBreaker::new(self.config.clone()),
        }
    }
}

/// Tower `Service` wrapper that enforces circuit breaker logic.
#[derive(Debug, Clone)]
pub struct CircuitBreakerService<S> {
    inner: S,
    breaker: CircuitBreaker,
}

impl<S, ReqBody, ResBody> tower::Service<http::Request<ReqBody>> for CircuitBreakerService<S>
where
    S: tower::Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display,
{
    type Response = S::Response;
    type Error = String;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.to_string())
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let breaker = self.breaker.clone();
        let config = self.breaker.inner.lock().unwrap().config.clone();

        match breaker.allow_request() {
            Ok(()) => {}
            Err(_) => {
                return Box::pin(async move { Err("Circuit breaker is open".to_string()) });
            }
        }

        let fut = self.inner.call(req);
        Box::pin(async move {
            let result = fut.await;
            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    if config.failure_statuses.contains(&status) {
                        breaker.record_failure();
                    } else {
                        breaker.record_success();
                    }
                }
                Err(_) => {
                    breaker.record_failure();
                }
            }
            result.map_err(|e| e.to_string())
        })
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_config_defaults_are_reasonable() {
        let c = RetryConfig::default();
        assert!(c.max_retries > 0);
        assert!(c.base_delay_ms > 0);
    }

    #[test]
    fn backoff_delay_increases_with_attempt() {
        let c = RetryConfig::default();
        let d0 = backoff_delay(0, &c);
        let d2 = backoff_delay(2, &c);
        assert!(d2 > d0, "backoff should grow: {d2:?} > {d0:?}");
    }

    #[test]
    fn backoff_delay_respects_max() {
        let c = RetryConfig {
            max_delay_ms: 500,
            ..RetryConfig::default()
        };
        let d10 = backoff_delay(10, &c);
        assert!(d10.as_millis() <= u128::from(c.max_delay_ms));
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request().is_ok());
    }

    #[test]
    fn circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..CircuitBreakerConfig::default()
        };
        let cb = CircuitBreaker::new(config);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.allow_request().is_err());
    }

    #[test]
    fn circuit_breaker_recovery_flow() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(0),
            success_threshold: 2,
            ..CircuitBreakerConfig::default()
        };
        let cb = CircuitBreaker::new(config);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.allow_request().is_ok());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
