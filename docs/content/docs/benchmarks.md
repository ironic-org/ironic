---
title: Benchmarks
description: Reproducible startup, registration, DI, and request overhead measurements.
---

# Benchmarks

Run the dependency-free comparative harness with:

```bash
cargo bench -p rustframe --bench overhead
```

Measurements from 2026-07-13 on Darwin 25.5.0 arm64, Rust 1.85.0:

| Operation | Time |
|---|---:|
| Module graph compilation | 866 ns/op |
| Route registration | 436 ns/op |
| Transient provider resolution | 157 ns/op |
| HTTP runtime startup | 555 ns/op |
| RustFrame in-process request | 780 ns/op |
| Raw Axum in-process request | 319 ns/op |

These are single-machine, in-memory release-build measurements, not capacity claims. The request
comparison includes RustFrame route lookup, request-ID/tracing middleware, controller resolution,
pipeline execution, and response conversion. Re-run on deployment hardware and benchmark real
handlers before setting performance budgets.
