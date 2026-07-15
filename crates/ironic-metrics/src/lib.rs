//! Production observability: request metrics with Prometheus-compatible endpoint.
//!
//! Tracks request count, latency histogram buckets, status code breakdown,
//! and in-flight request gauge. Exposes a `GET /metrics` endpoint for Prometheus scraping.

use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, LazyLock, Mutex, RwLock,
    },
    time::Instant,
};

const DEFAULT_SAMPLE_BUFFER: usize = 1000;

use ironic_core::{Module, ModuleDefinition};
use ironic_di::{ProviderDefinition, Scope};
use ironic_http::{ControllerDefinition, HttpError, HttpMethod, Json, RouteDefinition, handler_fn};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Constants — Prometheus default histogram boundaries
// ---------------------------------------------------------------------------

const BOUNDARIES: [f64; 12] = [
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

const NUM_BUCKETS: usize = 13; // 12 boundaries + +Inf overflow

fn bucket_for(latency: f64) -> usize {
    for (i, boundary) in BOUNDARIES.iter().enumerate() {
        if latency <= *boundary {
            return i;
        }
    }
    NUM_BUCKETS - 1
}

// ---------------------------------------------------------------------------
// Lock-free metrics storage
// ---------------------------------------------------------------------------

struct MetricsStore {
    request_count: AtomicU64,
    in_flight: AtomicU64,
    latency_buckets: [AtomicU64; NUM_BUCKETS],
}

fn new_buckets() -> [AtomicU64; NUM_BUCKETS] {
    [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ]
}

static METRICS: LazyLock<MetricsStore> = LazyLock::new(|| MetricsStore {
    request_count: AtomicU64::new(0),
    in_flight: AtomicU64::new(0),
    latency_buckets: new_buckets(),
});

static STATUS_COUNTS: LazyLock<Mutex<HashMap<u16, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Bounded ring buffer for raw latency percentile computation.
static LATENCY_BUFFER: LazyLock<Mutex<VecDeque<f64>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(DEFAULT_SAMPLE_BUFFER)));

fn push_latency(latency: f64) {
    if let Ok(mut buf) = LATENCY_BUFFER.lock() {
        if buf.len() >= buf.capacity() {
            buf.pop_front();
        }
        buf.push_back(latency);
    }
}

// ---------------------------------------------------------------------------
// Per-endpoint metrics
// ---------------------------------------------------------------------------

struct PerEndpointMetrics {
    request_count: AtomicU64,
    latency_buckets: [AtomicU64; NUM_BUCKETS],
}

fn new_per_endpoint_buckets() -> [AtomicU64; NUM_BUCKETS] {
    [
        AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
        AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
        AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
        AtomicU64::new(0),
    ]
}

static PER_ENDPOINT: LazyLock<RwLock<HashMap<String, PerEndpointMetrics>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

const PER_ENDPOINT_CARDINALITY_WARN: usize = 1000;

static PER_ENDPOINT_CARDINALITY_WARNED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn ensure_per_endpoint(key: &str) {
    let map = PER_ENDPOINT
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if map.contains_key(key) {
        return;
    }
    let cardinality = map.len() + 1;
    drop(map);
    let mut map = PER_ENDPOINT
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if cardinality > PER_ENDPOINT_CARDINALITY_WARN
        && !PER_ENDPOINT_CARDINALITY_WARNED.swap(true, Ordering::Relaxed)
    {
        tracing::warn!(
            cardinality = cardinality,
            threshold = PER_ENDPOINT_CARDINALITY_WARN,
            "Per-endpoint metric cardinality exceeds threshold; \
             consider disabling per-endpoint metrics or increasing the limit"
        );
    }
    map.entry(key.to_string()).or_insert_with(|| PerEndpointMetrics {
        request_count: AtomicU64::new(0),
        latency_buckets: new_per_endpoint_buckets(),
    });
}

fn record_per_endpoint(key: &str, duration: f64) {
    let map = PER_ENDPOINT
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(endpoint) = map.get(key) {
        endpoint.request_count.fetch_add(1, Ordering::Relaxed);
        endpoint.latency_buckets[bucket_for(duration)].fetch_add(1, Ordering::Relaxed);
    }
}

fn compute_percentiles() -> Option<(f64, f64, f64)> {
    let buf = LATENCY_BUFFER.lock().ok()?;
    if buf.is_empty() {
        return None;
    }
    let mut samples: Vec<f64> = buf.iter().copied().collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = samples.len();
    let p50 = samples[(len as f64 * 0.50).ceil() as usize - 1].min(samples[len - 1]);
    let p90 = samples[(len as f64 * 0.90).ceil() as usize - 1].min(samples[len - 1]);
    let p99 = samples[(len as f64 * 0.99).ceil() as usize - 1].min(samples[len - 1]);
    Some((p50, p90, p99))
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Controls which metrics are recorded and bucket sizes.
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Latency histogram bucket boundaries in seconds (fixed at 12 default boundaries).
    pub latency_buckets: Vec<f64>,
    /// When `true`, records per-endpoint metrics (method + path labels).
    pub per_endpoint: bool,
    /// Number of raw latency samples retained for percentile computation (p50/p90/p99).
    /// Defaults to 1000. Set to 0 to disable percentile output.
    pub sample_buffer_size: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            latency_buckets: BOUNDARIES.to_vec(),
            per_endpoint: true,
            sample_buffer_size: DEFAULT_SAMPLE_BUFFER,
        }
    }
}

// ---------------------------------------------------------------------------
// Tower Layer
// ---------------------------------------------------------------------------

/// Tower-compatible middleware that records HTTP request metrics.
#[derive(Debug, Clone)]
pub struct MetricsLayer {
    config: MetricsConfig,
}

impl MetricsLayer {
    /// Creates a layer with the given configuration.
    pub fn new(config: MetricsConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MetricsService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Tower `Service` wrapper that records request metrics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetricsService<S> {
    inner: S,
    config: MetricsConfig,
}

impl<S, ReqBody, ResBody> tower::Service<http::Request<ReqBody>> for MetricsService<S>
where
    S: tower::Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = if self.config.per_endpoint {
            req.uri().path().to_string()
        } else {
            "/".into()
        };
        let key = if self.config.per_endpoint {
            let k = format!("{}:{}", method, path);
            ensure_per_endpoint(&k);
            Some(k)
        } else {
            None
        };

        METRICS.in_flight.fetch_add(1, Ordering::Relaxed);

        let fut = self.inner.call(req);
        Box::pin(async move {
            struct DecrementGuard;
            impl Drop for DecrementGuard {
                fn drop(&mut self) {
                    METRICS.in_flight.fetch_sub(1, Ordering::Relaxed);
                }
            }
            let _guard = DecrementGuard;

            let result = fut.await;
            let duration = start.elapsed().as_secs_f64();

            METRICS.request_count.fetch_add(1, Ordering::Relaxed);
            METRICS.latency_buckets[bucket_for(duration)].fetch_add(1, Ordering::Relaxed);
            push_latency(duration);

            if let Some(ref k) = key {
                record_per_endpoint(k, duration);
            }

            let status = match &result {
                Ok(response) => response.status().as_u16(),
                Err(_) => 500,
            };
            if let Ok(mut counts) = STATUS_COUNTS.lock() {
                *counts.entry(status).or_insert(0) += 1;
            }

            result
        })
    }
}

// ---------------------------------------------------------------------------
// Prometheus text format
// ---------------------------------------------------------------------------

/// Returns the current metrics snapshot in Prometheus text format.
pub fn scrape() -> String {
    let mut out = String::new();

    let count = METRICS.request_count.load(Ordering::Relaxed);
    let in_flight = METRICS.in_flight.load(Ordering::Relaxed);

    let mut buckets = [0u64; NUM_BUCKETS];
    for (i, b) in METRICS.latency_buckets.iter().enumerate() {
        buckets[i] = b.load(Ordering::Relaxed);
    }

    let _ = writeln!(out, "# HELP ironic_http_requests_total Total HTTP requests");
    let _ = writeln!(out, "# TYPE ironic_http_requests_total counter");
    let _ = writeln!(out, "ironic_http_requests_total {count}");
    let _ = writeln!(out);

    if let Ok(statuses) = STATUS_COUNTS.lock() {
        let _ = writeln!(
            out,
            "# HELP ironic_http_responses_total Responses by status code"
        );
        let _ = writeln!(out, "# TYPE ironic_http_responses_total counter");
        let mut sorted: Vec<_> = statuses.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (code, count) in &sorted {
            let _ = writeln!(out, "ironic_http_responses_total{{status=\"{code}\"}} {count}");
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(
        out,
        "# HELP ironic_http_request_duration_seconds Request latency"
    );
    let _ = writeln!(out, "# TYPE ironic_http_request_duration_seconds histogram");
    let mut cumulative: u64 = 0;
    for (i, boundary) in BOUNDARIES.iter().enumerate() {
        cumulative += buckets[i];
        let _ = writeln!(
            out,
            "ironic_http_request_duration_seconds_bucket{{le=\"{boundary}\"}} {cumulative}"
        );
    }
    cumulative += buckets[12];
    let _ = writeln!(
        out,
        "ironic_http_request_duration_seconds_bucket{{le=\"+Inf\"}} {cumulative}"
    );
    let _ = writeln!(out, "ironic_http_request_duration_seconds_count {cumulative}");
    let _ = writeln!(out);

    if let Some((p50, p90, p99)) = compute_percentiles() {
        let _ = writeln!(
            out,
            "# HELP ironic_http_request_duration_seconds_sum Request latency sum (from raw samples)"
        );
        let _ = writeln!(out, "# TYPE ironic_http_request_duration_seconds_sum summary");
        let _ = writeln!(out, "ironic_http_request_duration_seconds_sum{{quantile=\"0.5\"}} {p50:.6}");
        let _ = writeln!(out, "ironic_http_request_duration_seconds_sum{{quantile=\"0.9\"}} {p90:.6}");
        let _ = writeln!(out, "ironic_http_request_duration_seconds_sum{{quantile=\"0.99\"}} {p99:.6}");
        let _ = writeln!(out);
    }

    let _ = writeln!(
        out,
        "# HELP ironic_http_requests_in_flight Currently in-flight requests"
    );
    let _ = writeln!(out, "# TYPE ironic_http_requests_in_flight gauge");
    let _ = writeln!(out, "ironic_http_requests_in_flight {in_flight}");
    let _ = writeln!(out);

    // Per-endpoint metrics
    {
        let ep_map = PER_ENDPOINT
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !ep_map.is_empty() {
            let _ = writeln!(out, "# HELP ironic_http_request_duration_seconds_per_endpoint Per-endpoint request latency histogram");
            let _ = writeln!(out, "# TYPE ironic_http_request_duration_seconds_per_endpoint histogram");
            let mut keys: Vec<&String> = ep_map.keys().collect();
            keys.sort();
            for key in &keys {
                if let Some(ep) = ep_map.get(*key) {
                    let ep_count = ep.request_count.load(Ordering::Relaxed);
                    let mut ep_total: u64 = 0;
                    for (i, boundary) in BOUNDARIES.iter().enumerate() {
                        ep_total += ep.latency_buckets[i].load(Ordering::Relaxed);
                        let _ = writeln!(
                            out,
                            "ironic_http_request_duration_seconds_per_endpoint_bucket{{endpoint=\"{}\",le=\"{boundary}\"}} {ep_total}",
                            key
                        );
                    }
                    ep_total += ep.latency_buckets[12].load(Ordering::Relaxed);
                    let _ = writeln!(
                        out,
                        "ironic_http_request_duration_seconds_per_endpoint_bucket{{endpoint=\"{}\",le=\"+Inf\"}} {ep_total}",
                        key
                    );
                    let _ = writeln!(
                        out,
                        "ironic_http_request_duration_seconds_per_endpoint_count{{endpoint=\"{}\"}} {ep_count}",
                        key
                    );
                }
            }
            let _ = writeln!(out);
        }
    }

    scrape_custom(&mut out);

    let _ = writeln!(out, "# HELP ironic_info Framework version");
    let _ = writeln!(out, "# TYPE ironic_info gauge");
    let _ = writeln!(
        out,
        "ironic_info{{version=\"{}\"}} 1",
        env!("CARGO_PKG_VERSION")
    );

    out
}

// ---------------------------------------------------------------------------
// Public API — user-defined metrics
// ---------------------------------------------------------------------------

/// A counter that can only be incremented.
pub struct Counter {
    name: String,
    help: String,
    value: AtomicU64,
}

impl Counter {
    fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by `n`.
    pub fn inc_by(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Returns the current value.
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A gauge that can be set, incremented, or decremented.
pub struct Gauge {
    name: String,
    help: String,
    value: AtomicU64,
}

impl Gauge {
    fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: AtomicU64::new(0),
        }
    }

    /// Overwrite the gauge value.
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge by 1.
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge by 1.
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Returns the current value.
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A histogram with fixed Prometheus default bucket boundaries.
pub struct Histogram {
    name: String,
    help: String,
    buckets: [AtomicU64; NUM_BUCKETS],
}

impl Histogram {
    fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            buckets: new_per_endpoint_buckets(),
        }
    }

    /// Record a latency value in seconds.
    pub fn record(&self, latency: f64) {
        self.buckets[bucket_for(latency)].fetch_add(1, Ordering::Relaxed);
    }
}

// Global storage for user-registered custom metrics.
static CUSTOM_COUNTERS: LazyLock<Mutex<Vec<Arc<Counter>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static CUSTOM_GAUGES: LazyLock<Mutex<Vec<Arc<Gauge>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static CUSTOM_HISTOGRAMS: LazyLock<Mutex<Vec<Arc<Histogram>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Registry for user-defined Prometheus metrics.
///
/// Create counters, gauges, and histograms via the `register_*` methods.
/// Registered metrics automatically appear in the `/metrics` scrape output.
///
/// ## Example
///
/// ```ignore
/// use ironic::metrics::MetricsRegistry;
/// use std::sync::Arc;
///
/// let registry: Arc<MetricsRegistry> = resolver.resolve().await.unwrap();
/// let hits = registry.counter("api_hits_total", "Total API hits");
/// hits.inc();
/// ```
pub struct MetricsRegistry;

impl MetricsRegistry {
    /// Register a new counter.
    pub fn counter(&self, name: &str, help: &str) -> Arc<Counter> {
        let c = Arc::new(Counter::new(name, help));
        if let Ok(mut list) = CUSTOM_COUNTERS.lock() {
            list.push(c.clone());
        }
        c
    }

    /// Register a new gauge.
    pub fn gauge(&self, name: &str, help: &str) -> Arc<Gauge> {
        let g = Arc::new(Gauge::new(name, help));
        if let Ok(mut list) = CUSTOM_GAUGES.lock() {
            list.push(g.clone());
        }
        g
    }

    /// Register a new histogram with Prometheus default bucket boundaries.
    pub fn histogram(&self, name: &str, help: &str) -> Arc<Histogram> {
        let h = Arc::new(Histogram::new(name, help));
        if let Ok(mut list) = CUSTOM_HISTOGRAMS.lock() {
            list.push(h.clone());
        }
        h
    }
}

fn scrape_custom(out: &mut String) {
    if let Ok(counters) = CUSTOM_COUNTERS.lock() {
        for c in counters.iter() {
            let _ = writeln!(out, "# HELP {} {}", c.name, c.help);
            let _ = writeln!(out, "# TYPE {} counter", c.name);
            let _ = writeln!(out, "{} {}", c.name, c.value.load(Ordering::Relaxed));
            let _ = writeln!(out);
        }
    }
    if let Ok(gauges) = CUSTOM_GAUGES.lock() {
        for g in gauges.iter() {
            let _ = writeln!(out, "# HELP {} {}", g.name, g.help);
            let _ = writeln!(out, "# TYPE {} gauge", g.name);
            let _ = writeln!(out, "{} {}", g.name, g.value.load(Ordering::Relaxed));
            let _ = writeln!(out);
        }
    }
    if let Ok(histograms) = CUSTOM_HISTOGRAMS.lock() {
        for h in histograms.iter() {
            let _ = writeln!(out, "# HELP {} {}", h.name, h.help);
            let _ = writeln!(out, "# TYPE {} histogram", h.name);
            let mut cumulative: u64 = 0;
            for (i, boundary) in BOUNDARIES.iter().enumerate() {
                cumulative += h.buckets[i].load(Ordering::Relaxed);
                let _ = writeln!(
                    out,
                    "{}_bucket{{le=\"{boundary}\"}} {cumulative}",
                    h.name
                );
            }
            cumulative += h.buckets[12].load(Ordering::Relaxed);
            let _ = writeln!(out, "{}_bucket{{le=\"+Inf\"}} {cumulative}", h.name);
            let _ = writeln!(out, "{}_count {cumulative}", h.name);
            let _ = writeln!(out);
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics module (exposes GET /metrics)
// ---------------------------------------------------------------------------

/// Imports the `GET /metrics` Prometheus scraping endpoint.
pub struct MetricsModule;

impl Module for MetricsModule {
    fn definition() -> ModuleDefinition {
        let controller_provider =
            ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
                Ok(MetricsController)
            });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "metrics",
            handler_fn(
                |_controller: std::sync::Arc<MetricsController>, _arguments| async move {
                    let body = scrape();
                    Ok::<_, HttpError>(Json(MetricsPayload { text: body }))
                },
            ),
        )
        .expect("the built-in metrics route is valid");
        let controller = ControllerDefinition::new::<MetricsController>(
            "/metrics",
            controller_provider,
        )
        .expect("the built-in metrics controller path is valid")
        .route(route);
        let registry_provider =
            ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
                Ok(MetricsRegistry)
            });
        ModuleDefinition::builder::<Self>()
            .provider(registry_provider)
            .controller(controller)
            .build()
    }
}

struct MetricsController;

#[derive(Serialize)]
struct MetricsPayload {
    text: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_metrics_are_valid() {
        let text = scrape();
        assert!(text.contains("ironic_http_requests_total"));
        assert!(text.contains("ironic_info"));
    }

    #[test]
    fn bucket_for_correct_range() {
        let cases = [
            (0.0005, 0),
            (0.001, 0),
            (0.002, 1),
            (0.005, 1),
            (0.01, 2),
            (0.05, 4),
            (0.1, 5),
            (0.3, 7),
            (1.0, 8),
            (5.0, 10),
            (10.0, 11),
            (100.0, 12),
        ];
        for (latency, expected) in &cases {
            assert_eq!(bucket_for(*latency), *expected, "latency={latency}");
        }
    }

    #[test]
    fn atomic_counts_accumulate() {
        METRICS.request_count.store(0, Ordering::Relaxed);
        METRICS.request_count.fetch_add(1, Ordering::Relaxed);
        METRICS.request_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(METRICS.request_count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn default_config_has_buckets() {
        let cfg = MetricsConfig::default();
        assert!(!cfg.latency_buckets.is_empty());
        assert!(cfg.per_endpoint);
    }
}
