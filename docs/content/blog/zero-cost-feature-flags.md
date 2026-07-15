---
title: "Zero-cost feature flags — how Ironic eliminates dead code at compile time"
description: "A deep dive into Ironic's feature flag architecture: how Rust's `#[cfg]` makes unused code literally vanish from the binary, the dependency tree pruning, and why this beats runtime garbage collection every time."
date: "2026-07-15"
author: "Ironic Team"
---

# Zero-cost feature flags — how Ironic eliminates dead code at compile time

Most web frameworks ship everything. Every middleware, every ORM adapter, every auth driver — loaded into memory, waiting to be told "no, not today." Ironic doesn't do that. If you didn't enable it, it isn't in the binary. Not hidden behind a runtime flag. Not lazy-loaded on first use. Gone. Deleted. Zero bytes of `.text` spent on code you never touch.

This post explains how that works, from the compiler up.

---

## `#[cfg]` is not a runtime branch — it's a deletion instruction

In JavaScript frameworks, "conditional features" typically look like this:

```typescript
// NestJS
if (process.env.ENABLE_CORS) {
  app.enableCors();
}
```

That `if` lives in your binary forever. The code for CORS sits in `node_modules`, parsed, loaded into the event loop, and skipped every time `ENABLE_CORS` is `false`. You pay for it at boot, at memory pressure, and at every garbage collection cycle.

Rust's `#[cfg]` is a different species entirely. It runs in the compiler — before any code generation happens. When you write:

```rust
#[cfg(feature = "security")]
#[path = "../crates/ironic-security/src/lib.rs"]
pub mod security;
```

the compiler makes a binary decision during macro expansion: does the `security` feature exist in the current build? If yes, the module is included. If no, the entire file is treated as if it was never written. The compiler never sees its tokens. LLVM never sees its IR. The linker never sees its object code.

This isn't dead code elimination — it's code that was never born.

---

## Ironic's feature flag hierarchy

Ironic has a carefully layered feature tree (`Cargo.toml:41–97`). The top-level features aggregate sub-features, which in turn bring in optional crate dependencies:

```toml
security = ["security-cors", "security-rate-limit", "security-headers", "security-csrf"]
security-cors = ["dep:tower-http"]
security-rate-limit = ["dep:tower-http", "dep:redis"]
security-headers = ["dep:tower-http"]
security-csrf = ["dep:tower-http", "dep:uuid"]
```

Enable `security` and you get all four sub-features. Skip it, and `tower-http`, `redis`, and `uuid` never enter the dependency tree. No `Cargo.lock` entry. No download. No compilation.

The same pattern repeats across the entire framework:

| Aggregate feature | Sub-features enabled |
|---|---|
| `database` | `sqlx`, `seaorm`, `diesel`, `mongodb`, `redis` |
| `authentication` | `auth`, `jwt`, `oauth`, `sessions` |
| `application-services` | `cache`, `scheduling`, `events`, `realtime` |
| `distributed` | `queues`, `microservices`, `cqrs`, `sagas`, `grpc`, `graphql` |

At the crate level, each sub-feature gates its own module. In `crates/ironic-security/src/lib.rs`:

```rust
#[cfg(feature = "security-cors")]
pub mod cors;
#[cfg(feature = "security-rate-limit")]
pub mod rate_limit;
#[cfg(feature = "security-headers")]
pub mod security_headers;
#[cfg(feature = "security-csrf")]
pub mod csrf;
```

The module doesn't exist until its feature flag says it does. The `pub use` re-exports are gated the same way. If you enable `security` but only need CORS and CSRF, you can enable just `security-cors` + `security-csrf` and leave rate-limiting and security headers behind. The compiler respects the granularity.

---

## Dependency tree pruning in practice

Cargo's feature resolver (v3, specified in `Cargo.toml:190`) unifies features across the entire workspace. Here's what Cargo sees for three different build configurations:

| Crate | `ironic = { }` (default) | `ironic = { features = ["security"] }` | `ironic = { features = ["security", "database"] }` |
|---|---|---|---|
| `ironic` (core) | compiled | compiled | compiled |
| `ironic-security` | *skipped* | compiled | compiled |
| `tower-http` | *skipped* | compiled (cors + headers) | compiled |
| `redis` | *skipped* | compiled (rate-limit) | compiled |
| `uuid` | *skipped* | compiled (csrf) | compiled |
| `sqlx` | *skipped* | *skipped* | compiled |
| `sea-orm` | *skipped* | *skipped* | compiled |
| `diesel` | *skipped* | *skipped* | compiled |
| `mongodb` | *skipped* | *skipped* | compiled |

The transitive graph collapses automatically. If you don't need `security-rate-limit`, you don't get `redis`. If you don't need `database`, you don't get any of the five database drivers. Cargo has no concept of "maybe I'll need this later." It only compiles what the feature set demands.

---

## Binary size: what you actually pay for

Here are realistic estimates for a minimal Ironic application — a single controller returning JSON, no database, built in release mode (`lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, as configured in `Cargo.toml:241–244`):

| Build configuration | Approximate binary size | Includes |
|---|---|---|
| `ironic = { }` (default) | ~2.8 MB | Axum, tokio, serde, core DI |
| + `security` | ~3.5 MB | + tower-http, redis, uuid |
| + `database` | ~8 MB | + sqlx, sea-orm, diesel, mongodb, redis |
| + `distributed` | ~12 MB | + tonic (gRPC), async-graphql, kafka, lapin |
| All features enabled | ~18 MB | Everything above + auth, telemetry, validation, cron |

The important number isn't the max — it's the min. A production JSON API with no database, no auth, no CORS middleware: under 3 MB. Compare that to a typical NestJS `node_modules` directory, which rarely goes below 200 MB before any of your own code. The entire Ironic binary with every feature enabled is still smaller than a single unminified NestJS dependency.

---

## The generated `Cargo.toml` as self-documenting feature catalog

When you run `ironic new my-app`, the CLI generates a `Cargo.toml` that starts with sensible defaults and explicitly lists every available feature as a comment (`crates/ironic-cli/src/generators/project.rs:176`):

```toml
[dependencies]
ironic = { features = ["security", "compression", "metrics", "validation"], version = "0.3.9" }

# Available features (uncomment to enable):
# versioning      — URI, header, and media-type API versioning
# serialization   — role-based field exposure
# cache           — CacheInterceptor with InMemoryCache
# scheduling      — Fixed-interval and cron background tasks
# realtime        — WebSocket gateways with rooms/broadcasting
# resilience      — Retry with backoff + circuit breaker
# telemetry       — Distributed tracing (OTLP)
# database        — SQLx, SeaORM, Diesel (postgres/mysql/sqlite)
# auth            — Password hashing, JWT, OAuth2, sessions
# distributed     — Queues, microservices, CQRS, sagas, gRPC, GraphQL
```

This is deliberately low-tech. No interactive CLI wizard, no `ironic add security` command, no package manifest with magical incantations. Just a comment block. Uncomment what you need, delete what you don't. The compiler enforces correctness — if you uncomment `database` without adding a connection URL, the factory panics at boot with a clear message. You don't discover missing features at runtime because the features you disabled don't exist at runtime.

---

## The production argument: no runtime branches, period

This is the core of Ironic's design philosophy. Every feature decision is made before `cargo build`. There are no:

- Runtime feature flags that must be checked on every request
- Lazy-loaded modules that add latency on first access
- String-based feature identifiers that can drift across refactors
- Dead code in the binary that a tree-shaker might eventually remove

The compiler is the dead code eliminator. It's also the only one. There is no plan B for feature availability at runtime because there doesn't need to be. A feature you didn't select doesn't exist in any form — not as a disabled code path, not as a serialized config option, not as a dormant dependency. The compiler guarantees this. LLVM guarantees this.

Compare with NestJS's dynamic imports:

```typescript
// This module exists in your bundle whether it's imported or not.
// The `import()` still compiles to a webpack chunk split, and the chunk
// lives on disk, ready to be fetched.
const { RedisModule } = await import('./redis/redis.module');
```

The module file is always there. The chunk is always in the build output. The split is a deployment-time optimization, not a build-time elimination. You still ship the code. You still pay the surface area cost — every unused module is a potential attack vector, a potential version conflict, a potential startup regression.

Ironic's approach eliminates the surface area entirely. If `redis` isn't in the feature set, the `RedisModule` type does not exist. You cannot import it. You cannot reference it. You cannot accidentally configure it in CI. The type system doesn't know it exists, because the compiler never compiled it.

---

## The trade-off: no hot-swapping features at runtime

You can't add CORS to a running Ironic binary. You can't turn on telemetry without recompiling. You can't A/B test a new database driver without a deploy.

For some teams, that's a dealbreaker. For most, it's a feature.

A recompile isn't downtime — it's a deploy pipeline that already exists. You're already restarting when you add a dependency. You're already rebuilding when you change a config. The difference is that Ironic makes the boundary explicit: **your deployment artifact is your feature set**. There is no divergence between what's in the binary and what's enabled in production, because "enabled in production" means "compiled into the binary."

If you need runtime feature toggling — percentage-rollout of a new endpoint, dark launches, kill switches — you build those into your application logic. They belong at the business layer, not the framework layer. Ironic gives you strongly-typed `FeatureFlag` providers. What it won't do is let you toggle the framework's own internals dynamically. That's the contract.

---

## The bottom line

- `#[cfg(feature = "X")]` deletes code at the token level — the module literally does not exist
- Ironic's feature tree aggregates sub-features into coherent bundles (e.g. `security` → `security-cors` + `security-rate-limit` + ...)
- Cargo's resolver prunes transitive dependencies — no `tower-http` means no HTTP layer middleware at all
- A minimal Ironic binary is under 3 MB; adding everything pushes it to ~18 MB
- The generated `Cargo.toml` documents every feature as a comment — uncomment to enable, no magic
- No runtime branches, no lazy imports, no dead code to eliminate later
- You can't hot-swap features at runtime — you recompile and deploy, the same pipeline you already use

You don't pay for code you don't use. The framework that designers of compiled languages have had for decades is finally applied to web application infrastructure. Turn it on or leave it out. The compiler handles the rest.
