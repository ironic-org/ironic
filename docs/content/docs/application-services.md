---
title: Application services
description: Caching, scheduling, typed events, WebSockets, and Server-Sent Events.
---

# Application services

Enable only the services an application uses, or select `application-services` for all of them.

- `cache` provides an asynchronous byte-cache contract, JSON helpers, TTL handling, and a bounded
  `InMemoryCache`. Implement `Cache` over Redis or another distributed backend in production.
- `scheduling` provides fixed-interval jobs with skipped missed ticks and cooperative shutdown.
- `events` provides a typed, bounded in-process event bus. Publishing applies backpressure.
- `realtime` exposes native Axum WebSocket upgrade types plus a bounded SSE channel. Mount realtime
  endpoints through Ironic's raw Axum router escape hatch.

Background tasks should be stopped from application lifecycle shutdown hooks. In-memory caches and
event buses are process-local and do not coordinate multiple replicas.
