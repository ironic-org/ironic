---
title: "#[ironic::main] — Tokio runtime bootstrapping via macro surgery"
description: "A deep dive into how Ironic's proc-macro attribute rewrites async fn main() into synchronous Tokio runtime bootstrapping code, removing the need for users to ever import or configure Tokio."
date: "2026-07-15"
author: "Ironic Team"
---

# `#[ironic::main]` — Tokio runtime bootstrapping via macro surgery

Every Rust async framework has to solve the same bootstrap problem: how do you turn a user-written `async fn main()` into a working Tokio runtime? Actix-web makes you bring your own `#[tokio::main]`. Axum expects you to wire it yourself. Ironic takes the opposite approach — your `main` function never mentions Tokio. Instead, the `#[ironic::main]` proc macro rewrites your function at compile time so the framework owns the runtime configuration entirely.

---

## What the user writes

From the user's perspective, starting an Ironic application is deceptively simple:

```rust
#[ironic::main]
async fn main() {
    let app = ironic::ApplicationBuilder::new()
        .root_module::<modules::RootModule>()
        .build()
        .await
        .unwrap();
    app.serve().await.unwrap();
}
```

No `#[tokio::main]`, no `tokio::runtime::Builder`, no tokio import at all. Just an `async fn main()` decorated with a single attribute. But what actually ships to the compiler is something very different.

---

## Macro internals: rewriting the AST

The macro lives in `ironic-macros` and is defined as a standard `proc_macro_attribute`. It receives the raw token stream, parses it into a `syn::ItemFn`, and performs a series of compile-time safety checks before mutating the syntax tree.

First, it rejects any attribute arguments — `#[ironic::main(something)]` is a compile error. Then it validates that the function is actually `async` — writing `#[ironic::main] fn main()` without the `async` keyword produces the error: `` `#[ironic::main]` requires an async function ``. Function parameters are also rejected, since an entry point with arguments doesn't make sense at the OS level.

Once validation passes, the macro performs the core transformation. It strips the `async` keyword from the function signature by setting `function.sig.asyncness = None`. Then it captures the original function body and wraps it inside a synchronous block:

```rust
function.block = Box::new(syn::parse_quote!({
    ::ironic::__private::block_on(async move #body)
}));
```

The `#body` token is the original block from the user's `async fn main`. The macro wraps it in `async move { ... }` (preserving any move semantics the original async function implied) and passes it to `::ironic::__private::block_on`. The result is a synchronous `main` function that the Rust compiler can use as a standard entry point.

---

## The before/after transformation

Here's what the macro actually generates. Before the macro runs:

```rust
#[ironic::main]
async fn main() {
    let app = /* build and serve */;
}
```

After the macro expands (conceptually):

```rust
fn main() {
    ::ironic::__private::block_on(async move {
        let app = /* build and serve */;
    });
}
```

The original `async fn` is gone. What the compiler sees is a plain synchronous `fn main()` that delegates to `block_on`, which owns the runtime lifecycle.

---

## `block_on()`: the runtime factory

The `block_on` function is defined in the `__private` module of the Ironic crate:

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Ironic failed to initialize its Tokio runtime")
        .block_on(future)
}
```

This is the only place in the entire framework where a Tokio runtime is created for the main entry point. It constructs a multi-thread runtime with all I/O and time drivers enabled — the same configuration you'd get from `#[tokio::main]` with `flavor = "multi_thread"`. The runtime blocks on the user's future, which drives the entire application lifecycle, and returns its output when the async work completes.

If the runtime fails to build (which is vanishingly rare on standard platforms), the `expect` message includes an Ironic-branded diagnostic so users can identify the failure context.

---

## Why the `__private` module?

The `#[doc(hidden)]` attribute and the module name `__private` signal a deliberate API boundary. This module is not part of Ironic's public API — it's the mechanism the macro generates code against. By routing all generated code through `__private`, the framework maintains two guarantees:

1. **No accidental public API surface**: If `block_on` were a public function, users could call it directly, bypassing the macro, inventing their own runtime configurations, and creating unsupported combinations. Making it hidden prevents this.

2. **Stable interface for code generation**: The macro can rely on `__private::block_on` existing at a known path without committing to exposing `block_on` as a consumer-facing function. The entire internal module can change across minor versions without breaking user code.

This pattern — a public macro generating code against a `#[doc(hidden)]` internal module — is common in Rust frameworks (serde uses `serde::__private` extensively). It gives the macro author complete control over the generated code surface while hiding implementation details from the documentation.

---

## The net effect

Ironic's approach to runtime bootstrapping means users never write `#[tokio::main]`, never import `tokio`, and never configure runtime thread counts or scheduler policies. The framework owns the runtime configuration, which is important because it also owns the async lifecycle of every provider, middleware, and background task in the application. If a user could swap the runtime for a single-threaded or current-thread variant, providers expecting `Send`-based multi-thread semantics would fail at mysterious points far from the actual misconfiguration.

By sinking the runtime creation into a compile-time code transformation, Ironic eliminates an entire class of configuration errors while keeping the user-facing syntax exactly as idiomatic as the rest of the Rust async ecosystem.
