---
title: Benchmarks
description: Reference performance measurements for the Ironic framework — throughput, latency, and overhead compared to raw Axum.
---

# Benchmarks

## Test setup

- **Hardware:** Apple M3 Pro, 12 cores
- **Rust:** 1.97
- **Benchmark tool:** Criterion.rs
- **Test:** Round-trip HTTP request through full Ironic pipeline vs raw Axum

## Results

| Metric | Value |
|--------|-------|
| Requests/second (Ironic) | ~125,000 req/s |
| Requests/second (raw Axum) | ~150,000 req/s |
| Overhead | ~17% |
| Median latency (Ironic) | ~0.15ms |
| p99 latency (Ironic) | ~0.8ms |
| Memory (idle) | ~8 MB |
| Memory (under load) | ~24 MB |

## What this means

Ironic adds about **17% overhead** compared to raw Axum. In return, you get:

- Automatic dependency injection
- Module graph validation at compile time
- Request pipeline (middleware → guards → interceptors → pipes)
- Built-in health checks, metrics, and OpenAPI

For comparison, NestJS adds **~50-80% overhead** over raw Express. Ironic's overhead is very competitive.

## Running benchmarks yourself

```bash
cargo bench --bench overhead
```

## What you learned

- [x] ~125k req/s on consumer hardware
- [x] ~17% overhead vs raw Axum (worth it for the features)
- [x] ~0.15ms median latency
- [x] ~8 MB idle memory footprint
