---
title: Queues and distributed architecture
description: Queues, transports, gRPC, CQRS, sagas, and GraphQL feature modules.
---

# Queues and distributed architecture

The `distributed` feature enables all APIs in this section; each can also be selected separately.

- `queues`: the `Queue` contract and bounded `InMemoryQueue` with acknowledgement/requeue APIs.
- `microservices`: transport-neutral envelopes and connected in-memory duplex endpoints.
- `grpc`: the upstream Tonic API plus DI registration for reusable channels.
- `cqrs`: a typed command/query dispatcher that validates duplicate and missing handlers.
- `sagas`: ordered forward steps with reverse compensation after failure.
- `graphql`: the upstream async-graphql API and schema DI registration.

The in-memory queue and channel transport are deterministic development/test implementations. Use a
durable broker adapter for production delivery guarantees. Application message IDs, idempotency,
retry limits, dead-letter handling, tracing propagation, and schema evolution remain explicit
deployment decisions rather than hidden defaults.
