#![allow(clippy::type_complexity)]
//! Injectable HTTP client for inter-service communication with retry and
//! circuit breaker support.

use serde::de::DeserializeOwned;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

/// Returns the `traceparent` header value from the current tracing span.
fn traceparent_value() -> Option<String> {
    let span = tracing::Span::current();
    span.id().map(|id| {
        let trace_id = id.into_u64();
        format!("00-{trace_id:x}-{trace_id:x}-01")
    })
}

/// An injectable HTTP client for making requests to other services.
#[derive(Clone)]
pub struct HttpClientService;

impl HttpClientService {
    /// Creates a new HTTP client.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns a client with retry configuration for chaining.
    #[must_use]
    pub const fn with_retry(self, max_retries: u32, base_delay_ms: u64) -> RetryClient {
        RetryClient {
            max_retries,
            base_delay_ms,
        }
    }

    /// Returns a client with circuit breaker protection for chaining.
    #[must_use]
    pub fn with_circuit_breaker(self, name: &str) -> CircuitBreakerClient {
        CircuitBreakerClient::new(name)
    }

    /// Sends a GET request.
    pub async fn get<T: DeserializeOwned + Send + 'static>(
        &self,
        url: &str,
    ) -> Result<T, HttpClientError> {
        let url = url.to_string();
        let header_val = traceparent_value();
        tokio::task::spawn_blocking(move || {
            let mut req = ureq::get(&url);
            if let Some(v) = &header_val {
                req = req.header("traceparent", v);
            }
            let body = req.call()?.into_body().read_to_string()?;
            serde_json::from_str(&body).map_err(HttpClientError::Deserialize)
        })
        .await
        .map_err(|e| HttpClientError::Internal(e.to_string()))?
    }

    /// Sends a POST request with JSON body.
    pub async fn post<T: serde::Serialize + Send + 'static, R: DeserializeOwned + Send + 'static>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, HttpClientError> {
        let url = url.to_string();
        let body_json = serde_json::to_string(body).map_err(HttpClientError::Serialize)?;
        let header_val = traceparent_value();
        tokio::task::spawn_blocking(move || {
            let mut req = ureq::post(&url).header("Content-Type", "application/json");
            if let Some(v) = &header_val {
                req = req.header("traceparent", v);
            }
            let resp = req.send(&body_json)?;
            let body = resp.into_body().read_to_string()?;
            serde_json::from_str(&body).map_err(HttpClientError::Deserialize)
        })
        .await
        .map_err(|e| HttpClientError::Internal(e.to_string()))?
    }

    /// Sends a PUT request.
    pub async fn put<T: serde::Serialize + Send + 'static, R: DeserializeOwned + Send + 'static>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, HttpClientError> {
        let url = url.to_string();
        let body_json = serde_json::to_string(body).map_err(HttpClientError::Serialize)?;
        let header_val = traceparent_value();
        tokio::task::spawn_blocking(move || {
            let mut req = ureq::put(&url).header("Content-Type", "application/json");
            if let Some(v) = &header_val {
                req = req.header("traceparent", v);
            }
            let resp = req.send(&body_json)?;
            let body = resp.into_body().read_to_string()?;
            serde_json::from_str(&body).map_err(HttpClientError::Deserialize)
        })
        .await
        .map_err(|e| HttpClientError::Internal(e.to_string()))?
    }

    /// Sends a DELETE request.
    pub async fn delete(&self, url: &str) -> Result<(), HttpClientError> {
        let url = url.to_string();
        let header_val = traceparent_value();
        tokio::task::spawn_blocking(move || {
            let mut req = ureq::delete(&url);
            if let Some(v) = &header_val {
                req = req.header("traceparent", v);
            }
            req.call()?;
            Ok(())
        })
        .await
        .map_err(|e| HttpClientError::Internal(e.to_string()))?
    }
}

impl Default for HttpClientService {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RetryClient
// ---------------------------------------------------------------------------

/// A client wrapper that retries failed requests with exponential backoff.
pub struct RetryClient {
    max_retries: u32,
    base_delay_ms: u64,
}

impl RetryClient {
    async fn retry_inner<F, Fut, T>(&self, f: F) -> Result<T, HttpClientError>
    where
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, HttpClientError>> + Send,
    {
        let mut attempt = 0u32;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < self.max_retries => {
                    attempt += 1;
                    let delay = Duration::from_millis(self.base_delay_ms * u64::from(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Sends a GET request with retry.
    pub async fn get<T: DeserializeOwned + Send + 'static>(
        &self,
        url: &str,
    ) -> Result<T, HttpClientError> {
        let url = url.to_string();
        self.retry_inner(move || {
            let url = url.clone();
            async move {
                tokio::task::spawn_blocking(move || {
                    let body = ureq::get(&url)
                        .call()?
                        .into_body()
                        .read_to_string()?;
                    serde_json::from_str(&body).map_err(HttpClientError::Deserialize)
                })
                .await
                .map_err(|e| HttpClientError::Internal(e.to_string()))?
            }
        })
        .await
    }
}

// ---------------------------------------------------------------------------
// CircuitBreakerClient
// ---------------------------------------------------------------------------

/// Circuit breaker state machine.
#[derive(Clone, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// A client wrapper that protects downstream services with a circuit breaker.
pub struct CircuitBreakerClient {
    state: Arc<Mutex<BreakerState>>,
    failure_count: Arc<AtomicU64>,
    failure_threshold: u64,
    recovery_timeout: Duration,
    last_failure_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl CircuitBreakerClient {
    /// Creates a new circuit breaker client.
    #[must_use]
    pub fn new(_name: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(BreakerState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            last_failure_time: Arc::new(Mutex::new(None)),
        }
    }

    async fn call<F, Fut, T>(&self, f: F) -> Result<T, HttpClientError>
    where
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, HttpClientError>> + Send,
    {
        // Check circuit state before making the call
        let is_open = {
            let state = self.state.lock().unwrap();
            *state == BreakerState::Open
        };
        if is_open {
            let should_half_open = {
                let last_fail = self.last_failure_time.lock().unwrap();
                last_fail
                    .as_ref()
                    .map(|t| t.elapsed() > self.recovery_timeout)
                    .unwrap_or(false)
            };
            if should_half_open {
                let mut state = self.state.lock().unwrap();
                *state = BreakerState::HalfOpen;
            } else {
                return Err(HttpClientError::CircuitOpen);
            }
        }

        match f().await {
            Ok(result) => {
                let mut state = self.state.lock().unwrap();
                if *state == BreakerState::HalfOpen {
                    *state = BreakerState::Closed;
                }
                self.failure_count.store(0, Ordering::SeqCst);
                Ok(result)
            }
            Err(e) => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                *self.last_failure_time.lock().unwrap() = Some(std::time::Instant::now());
                if count >= self.failure_threshold {
                    let mut state = self.state.lock().unwrap();
                    if *state != BreakerState::Open {
                        *state = BreakerState::Open;
                    }
                }
                Err(e)
            }
        }
    }

    /// Sends a GET request with circuit breaker protection.
    pub async fn get<T: DeserializeOwned + Send + 'static>(
        &self,
        url: &str,
    ) -> Result<T, HttpClientError> {
        let url = url.to_string();
        self.call(move || {
            let url = url.clone();
            async move {
                tokio::task::spawn_blocking(move || {
                    let body = ureq::get(&url)
                        .call()?
                        .into_body()
                        .read_to_string()?;
                    serde_json::from_str(&body).map_err(HttpClientError::Deserialize)
                })
                .await
                .map_err(|e| HttpClientError::Internal(e.to_string()))?
            }
        })
        .await
    }
}

/// An error returned by [`HttpClientService`] operations.
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    /// The HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(#[from] ureq::Error),
    /// Serialization failed.
    #[error("Serialization failed: {0}")]
    Serialize(serde_json::Error),
    /// Response deserialization failed.
    #[error("Response deserialization failed: {0}")]
    Deserialize(serde_json::Error),
    /// Circuit breaker is open.
    #[error("Circuit breaker is open — downstream service unavailable")]
    CircuitOpen,
    /// Internal error (task spawn, etc.).
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retry_exhausts_attempts() {
        let client = HttpClientService::new();
        let retry = client.with_retry(2, 1);
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = std::sync::Arc::clone(&calls);
        let result: Result<i32, HttpClientError> = retry.retry_inner(move || {
            let c = std::sync::Arc::clone(&c);
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(HttpClientError::Internal("fail".into()))
            }
        }).await;
        assert!(result.is_err());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_succeeds_on_second_attempt() {
        let client = HttpClientService::new();
        let retry = client.with_retry(3, 1);
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = std::sync::Arc::clone(&calls);
        let result: Result<i32, HttpClientError> = retry.retry_inner(move || {
            let c = std::sync::Arc::clone(&c);
            async move {
                let attempt = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt == 0 {
                    Err(HttpClientError::Internal("fail".into()))
                } else {
                    Ok(42)
                }
            }
        }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
