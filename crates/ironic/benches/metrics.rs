//! Benchmarks measuring metrics recording overhead — both the built-in
//! `MetricsLayer` and custom `MetricsRegistry` operations.
//!
//! Run with: cargo bench --bench metrics --features metrics

use std::{hint::black_box, sync::Arc, time::Instant};

use axum::{Router, body::Body, http::Request, routing::get};
use ironic::metrics::{MetricsConfig, MetricsLayer, MetricsRegistry, scrape};
use tower::ServiceExt;

const ITERATIONS: u64 = 100_000;

fn main() {
    let registry = MetricsRegistry;

    // ── Counter operations ───────────────────────────────────────────

    let counter = registry.counter("bench_counter", "Benchmark counter");

    measure("Counter::inc()", ITERATIONS, || {
        counter.inc();
        black_box(());
    });
    measure("Counter::inc_by(5)", ITERATIONS, || {
        counter.inc_by(5);
        black_box(());
    });

    // ── Gauge operations ─────────────────────────────────────────────

    let gauge = registry.gauge("bench_gauge", "Benchmark gauge");

    measure("Gauge::set(42)", ITERATIONS, || {
        gauge.set(42);
        black_box(());
    });
    measure("Gauge::inc()", ITERATIONS, || {
        gauge.inc();
        black_box(());
    });
    measure("Gauge::dec()", ITERATIONS, || {
        gauge.dec();
        black_box(());
    });

    // ── Histogram operations ─────────────────────────────────────────

    let histogram = registry.histogram("bench_histogram", "Benchmark histogram");

    measure("Histogram::record(fast=0.003s)", ITERATIONS, || {
        histogram.record(0.003);
        black_box(());
    });
    measure("Histogram::record(slow=3.0s)", ITERATIONS, || {
        histogram.record(3.0);
        black_box(());
    });
    measure("Histogram::record(overflow=15.0s)", ITERATIONS, || {
        histogram.record(15.0);
        black_box(());
    });

    // ── Scrape overhead ──────────────────────────────────────────────

    measure("scrape() full output", 1_000, || {
        black_box(scrape());
    });

    // ── Concurrent counter operations ─────────────────────────────────

    let shared_counter = Arc::new(registry.counter("shared_counter", "Shared counter"));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let c = shared_counter.clone();
            std::thread::spawn(move || {
                for _ in 0..ITERATIONS / 8 {
                    c.inc();
                }
            })
        })
        .collect();
    let started = Instant::now();
    for t in threads {
        t.join().unwrap();
    }
    let elapsed = started.elapsed();
    let ns_per_op = elapsed.as_nanos() / u128::from(ITERATIONS);
    println!(
        "{:<40} {:>10} ns/op ({ITERATIONS} iterations across 8 threads)",
        "Counter::inc() concurrent (8 threads)", ns_per_op
    );

    // ── MetricsLayer request overhead ─────────────────────────────────

    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        // Without metrics layer (baseline)
        let raw = Router::new().route("/bench", get(|| async { "ok" }));
        let started = Instant::now();
        for _ in 0..ITERATIONS / 10 {
            black_box(
                raw.clone()
                    .oneshot(Request::get("/bench").body(Body::empty()).unwrap())
                    .await
                    .unwrap(),
            );
        }
        report("raw Axum request (no metrics)", ITERATIONS / 10, started);

        // With metrics layer
        let with_metrics = Router::new()
            .route("/bench", get(|| async { "ok" }))
            .layer(MetricsLayer::new(MetricsConfig::default()));
        let started = Instant::now();
        for _ in 0..ITERATIONS / 10 {
            black_box(
                with_metrics
                    .clone()
                    .oneshot(Request::get("/bench").body(Body::empty()).unwrap())
                    .await
                    .unwrap(),
            );
        }
        report("Axum request + MetricsLayer", ITERATIONS / 10, started);
    });
}

fn measure(label: &str, iterations: u64, mut operation: impl FnMut()) {
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    report(label, iterations, started);
}

fn report(label: &str, iterations: u64, started: Instant) {
    let elapsed = started.elapsed();
    let ns_per_op = elapsed.as_nanos() / u128::from(iterations);
    println!("{label:<40} {ns_per_op:>10} ns/op ({iterations} iterations)");
}
