---
title: Migration Guides
description: Upgrade guides covering breaking changes between Ironic release lines.
---

# Migration Guides

Ironic follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html) — breaking
changes land in minor (`v0.4.x`, `v1.0.x`) or patch releases marked in the changelog.
Each guide below lists the breaking changes and step-by-step migration steps.

> **Why this matters:** most upgrades are drop-in, but a handful of API renames and
> default-behavior changes need a few minutes of attention. Read the guide for the
> version you're upgrading **from** — each covers everything after the previous guide.

| Upgrade path | What changed |
|--------------|-------------|
| [v0.3.x → v1.0.x](/docs/migrations/v1.0.x) | `Framework*` type renames, removed lifecycle hooks, removed `NotFoundFilter` |
| [v1.0.x → v1.2.x](/docs/migrations/v1.2.x) | `#[event_handler]` auto-registers by default, transport providers, auto-validation pipe |
| [v0.2.x → v0.3.x](/docs/migrations/v0.3.x) | Rate-limit backend, metrics rename, session stores, composite health format |

Newer guides build on older ones — for example, upgrading from `v0.3.x` to `v1.2.x`
means applying the `v1.0.x` guide first, then the `v1.2.x` guide.

## General upgrade steps

```bash
# 1. Bump the dependency
cargo add ironic@1.2

# 2. Build and let the compiler surface renamed items
cargo build

# 3. Read the guide for your old version, applying each migration step
# 4. Run your test suite — in-process tests catch most regressions instantly
cargo test
```

## Need help?

If you run into an issue not covered here, check the [FAQ](/docs/more/faq) or
open an issue at https://github.com/ironic-org/ironic/issues.
