---
title: Benchmarks
description: Reference performance measurements for the Ironic framework — pipeline overhead compared to raw Axum.
---

# Benchmarks

## Test setup

- **Hardware:** Apple M3 Pro, 12 cores
- **Rust:** current stable
- **Benchmark tool:** dependency-free `cargo bench` microbenchmarks (see `crates/ironic/benches/`)
- **Test:** In-process request through the full Ironic pipeline vs raw Axum

## Results

Measured on the current workspace at `crates/ironic/benches/overhead.rs`:

| Metric | Ironic | Raw Axum |
|--------|--------|----------|
| In-process request | 920 ns/op | 333 ns/op |
| Module graph compilation | 924 ns/op | — |
| Route registration | 492 ns/op | — |
| Transient provider resolution | 138 ns/op | — |
| HTTP runtime startup | 592 ns/op | — |

> These are **in-process microbenchmarks** — they measure the framework pipeline
> without any network I/O, which dominates real-world throughput. On a real HTTP
> round-trip, the framework overhead is a small fraction of the total latency.

## What this means

The full Ironic pipeline adds roughly **2-3×** per-request work compared to a bare
Axum handler. In exchange you get:

- Automatic dependency injection
- Module graph validation at compile time
- Request pipeline (middleware → guards → interceptors → pipes)
- Built-in health checks, metrics, and OpenAPI
- DTO validation, serialization, and error mapping

Because the added work is on the order of **microseconds per request**, it is
negligible for typical HTTP workloads (millisecond-scale network latency). For
hot paths that need to be maximally lean, raw Axum handlers are always an option —
Ironic composes with them.

## Metrics recording overhead

The `metrics` benchmark measures `MetricsLayer` and `MetricsRegistry` overhead:

```bash
cargo bench --bench metrics --features metrics
```

## Running benchmarks yourself

```bash
cargo bench --bench overhead
```

## What you learned

- [x] Full Ironic pipeline: ~920 ns/op in-process
- [x] Raw Axum handler: ~333 ns/op — the framework adds microsecond-scale overhead
- [x] Pipeline overhead is negligible vs network I/O in real HTTP workloads
- [x] Microbenchmarks live in `crates/ironic/benches/` and are dependency-free
