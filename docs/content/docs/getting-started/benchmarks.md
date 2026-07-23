---
title: Benchmarks
description: Performance benchmarks comparing Ironic to raw Axum and other frameworks.
---

# Benchmarks

Ironic is designed to have minimal overhead over the underlying Axum runtime. Here are the latest benchmark results.

## Microbenchmarks

Measured with `cargo bench` on an M-series MacBook Pro:

| Operation | Time | Notes |
|-----------|------|-------|
| Empty route (raw Axum) | ~391 ns | Baseline |
| Ironic controller dispatch | ~550 ns | +159 ns overhead |
| With `MetricsLayer` | ~714 ns | Full observability pipeline |
| DI container resolve (singleton) | ~200 ns | Cached, first call only |
| DI container resolve (transient) | ~600 ns | New instance each time |
| Counter increment | <1 ns | AtomicU64 relaxed store |
| Gauge set | <1 ns | AtomicU64 relaxed store |
| Histogram record | ~1 ns | Bucket index + AtomicU64 increment |
| Metrics scrape (empty) | ~600 ns | Generates Prometheus text |
| Retry layer | ~50 ns | Per-request overhead |
| Circuit breaker (closed) | ~30 ns | Atomic state check |

## Latency overhead (p50)

```
Raw Axum:       391 ns
Ironic:         550 ns  (+40%)
Ironic + Metrics: 714 ns  (+82%)
```

In absolute terms, the overhead is **<1 microsecond** per request. For context:
- A typical database query takes **1-50ms**
- An external API call takes **50-500ms**
- Network latency is **1-100ms**

Ironic's overhead is invisible in real-world applications.

## Throughput

Preliminary throughput benchmarks (100 concurrent connections):

| Framework | Requests/sec | vs baseline |
|-----------|-------------|-------------|
| Raw Axum | ~125,000 | 1x |
| Ironic (no middleware) | ~110,000 | 0.88x |
| Ironic (full stack) | ~95,000 | 0.76x |

Throughput impact is proportional to the middleware pipeline. Most applications will be bottlenecked by database I/O or business logic, not the framework layer.

## Memory overhead

| Component | Memory |
|-----------|--------|
| Empty Ironic application | ~2 MB RSS |
| With 10 controllers + DI | ~5 MB RSS |
| With metrics + logging | ~8 MB RSS |
| With OpenAPI generation | ~12 MB RSS |

## Running benchmarks yourself

```bash
# Clone the repo
git clone https://github.com/ironic-org/ironic
cd ironic

# Run all benchmarks
cargo bench --all-features

# Run specific benchmarks
cargo bench --bench overhead
cargo bench --bench metrics
```

## Interpreting results

Ironic's overhead is concentrated in:
1. **DI resolution** — happens once per provider on first access
2. **Middleware dispatch** — tower layer overhead per request
3. **Metrics recording** — atomic counters per request

None of these are bottlenecks in real-world applications. The framework is designed for **production throughput** where the bottleneck is your business logic, not the framework.
