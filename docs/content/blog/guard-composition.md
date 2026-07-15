---
title: "Guard Composition — how multiple guards form a single access decision"
description: "A deep technical walkthrough of Ironic's guard system — how guards are ordered, evaluated sequentially, and how a single denial unwinds the entire middleware stack."
date: "2026-07-15"
author: "Ironic Team"
---

# Guard Composition — how multiple guards form a single access decision

Most frameworks let you bolt one or two guards onto a route and call it a day. Ironic generalizes the problem: what happens when you have _many_ guards, declared at different scopes, that all need to converge on a single allow/deny decision?

The answer lives in a 50-line function at `crates/ironic-http/src/pipeline.rs` and two traits. Here's the full model.

---

## The Guard trait

Every guard implements one method (`pipeline.rs:40-43`):

```rust
pub trait Guard: Send + Sync + 'static {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a>;
}
```

A guard receives a mutable `RequestContext` (it can read headers, inspect the route, pull from the DI container) and returns a `GuardFuture<'a>` — an async boxed future resolving to a `Result<GuardDecision, HttpError>`.

The two possible decisions are `GuardDecision::Allow` and `GuardDecision::Deny`. There's no "abstain" — every guard must commit. `Allow` means "I have no objection; proceed." `Deny` means "this request may not reach the handler."

Crucially, the return type is a `Result`. If a guard fails with an `Err`, that's an _error_ — not a denial. Errors propagate as 500s; denials become 403s. The distinction matters: a misconfigured guard or a broken DB connection should never be mistaken for an authorization failure.

---

## Sequential evaluation: all must Allow

Guards are not consulted in parallel. They run one at a time, in a strict order, inside `run_guards` (`pipeline.rs:273-296`):

```rust
fn run_guards<'a>(
    state: &'a ExecutionState<'a>,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    Box::pin(async move {
        let count = guard_count(state);
        for index in 0..count {
            match guard_at(state, index)
                .expect("guard index is in bounds")
                .can_activate(context)
                .await?
            {
                GuardDecision::Allow => {}
                GuardDecision::Deny => {
                    return Err(HttpError::forbidden(
                        "RF_HTTP_GUARD_DENIED",
                        "Access to this route was denied",
                    ));
                }
            }
        }
        run_interceptor(state, 0, context).await
    })
}
```

The loop iterates from index `0` to `count - 1`. Each guard's `can_activate` is awaited in turn. If the decision is `Allow`, execution moves to the next guard. If it's `Deny`, the function returns immediately with a `403 Forbidden` — no further guards are consulted, no handler runs, no interceptors fire.

---

## Denial unwinds middleware in reverse

When a guard denies, the control flow doesn't just stop — the `Result::Err` propagates _back through_ every middleware layer that was already entered, in reverse order. This is automatic: middleware runs `await next.call(context)` before it does its post-processing. When `next.call` returns an `Err` (because a guard deep in the stack denied), each middleware's post-request code still executes, sees the error, and can log, transform, or suppress it.

This means a guard denial triggers exactly the same unwind path as a handler panic — the entire middleware onion peels back, layer by layer, in LIFO order. The only difference is the HTTP status code embedded in the error.

---

## The guard index system: global before route

Not all guards are equal. Ironic lets you attach guards at two levels:

- **Application-global guards** (registered on the `Application` pipeline)
- **Route-level guards** (registered on a specific `Route`'s pipeline)

The index system enforces that global guards always run first, mirroring the exact same interleaving pattern used for middleware. Two functions control this (`pipeline.rs:349-358`):

```rust
fn guard_count(state: &ExecutionState<'_>) -> usize {
    state.application.pipeline().guards.len() + state.route.pipeline().guards.len()
}

fn guard_at<'a>(state: &'a ExecutionState<'a>, index: usize) -> Option<&'a Arc<dyn Guard>> {
    let global = &state.application.pipeline().guards;
    global
        .get(index)
        .or_else(|| state.route.pipeline().guards.get(index - global.len()))
}
```

`guard_count` sums both arrays. `guard_at` resolves an index: if `index < global.len()`, it pulls from the global guard list; otherwise it offsets into the route-level list. The `run_guards` loop iterates linearly from 0 to `count`, so the execution order is always **all global guards, then all route guards**.

There's no way to reorder them. Global guards cannot be bypassed by a route declaration. If you need a guard that always runs — authentication, rate-limiting — put it on the application. If you need a guard that only applies to `/admin/*`, put it on that route group.

---

## A concrete example: RoleGuard + ApiKeyGuard

Suppose your application declares a global `ApiKeyGuard` that validates an `X-API-Key` header against a known set of keys. Separately, your `/admin` route group declares a `RoleGuard` that checks for an `admin` role in a decoded JWT.

When a request hits `/admin/reports`:

1. **ApiKeyGuard** runs first (global). It checks the header. If the key is missing or invalid, it returns `Deny` → immediate 403, no further processing.
2. If the key is valid, ApiKeyGuard returns `Allow` → execution moves to `RoleGuard`.
3. **RoleGuard** runs second (route-level). It inspects the JWT claims. If the role is `admin`, it returns `Allow` → the handler fires. If the role is `user`, it returns `Deny` → 403, middleware unwinds.

Both guards must `Allow` for the request to reach the handler. The order matters: ApiKeyGuard rejects unauthenticated traffic before RoleGuard ever touches the JWT, saving a decode step for bad requests.

---

## Guard vs. middleware: the decision

A guard exists for one purpose: to say "no" and to do it early. A middleware exists to wrap, transform, augment, or observe the request around the handler call.

If you're checking a permission, use a guard. If you're adding a correlation ID to every response header, logging request duration, or gzipping the response body — that's middleware. Guards don't get a `next` callback because there _is_ no next for a denied request. Middleware always gets a `next`, and it can choose whether to call it.

The sequential evaluation model means you can stack guards with confidence: earlier guards screen out noise, later guards enforce domain-specific policies, and the entire stack collapses cleanly the moment any one says "no."

---

## Error propagation: errors are not denials

Notice the `?` after `can_activate(context).await` in `run_guards`. If a guard's future resolves to `Err(...)`, the question-mark operator propagates it directly — bypassing the `GuardDecision` match entirely. The error climbs back through the middleware stack as a `500 Internal Server Error`.

This preserves the semantics: a guard that cannot reach a decision (broken config, timed-out DB) _is not making a policy statement_. It's failing to function. The system treats it accordingly, and the caller knows the difference between "you're not allowed" (403) and "something went wrong" (500).

Guard errors also bubble through middleware in reverse, just like denials — every layer gets its chance to handle or log the error on the way out.
