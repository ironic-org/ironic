---
title: TCP Connection Limit
description: Limit open TCP connections to prevent file descriptor exhaustion from slow attacks.
---

# TCP Connection Limit

## What is it?

`AxumAdapter::max_connections()` caps the number of concurrent TCP connections. Without it, an attacker can exhaust file descriptors by opening thousands of connections and never sending data (slowloris attack).

## How to use

```rust
AxumAdapter::new()
    .max_connections(10_000)
    .max_concurrent_requests(5_000)
    .build();
```

## Connection limit vs Request limit

| Field | What it limits | Attack it prevents |
|-------|---------------|-------------------|
| `max_connections` | Open TCP sockets | Slowloris (exhaust file descriptors) |
| `max_concurrent_requests` | In-flight HTTP requests | Resource exhaustion (CPU/memory) |

## Recommended values

| Deployment | max_connections | max_concurrent_requests |
|-----------|----------------|------------------------|
| Small (1 vCPU) | 1,000 | 500 |
| Medium (4 vCPU) | 10,000 | 5,000 |
| Large (16 vCPU) | 50,000 | 25,000 |

## What you learned

- [x] `max_connections()` limits open TCP sockets per server instance
- [x] Combine with `max_concurrent_requests()` for defense in depth
- [x] Set based on available file descriptors (`ulimit -n`)
