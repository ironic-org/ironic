---
title: "Security Middleware Internals — four independent defense layers"
description: "A line-by-line dissection of Ironic's four built-in security middlewares — CORS, rate limiting, CSRF, and security headers — all sharing one trait but solving fundamentally different problems."
date: "2026-07-15"
author: "Ironic Team"
---

# Security Middleware Internals — four independent defense layers

Ironic ships four opt-in security middlewares, each behind its own feature flag: `security-cors`, `security-rate-limit`, `security-csrf`, and `security-headers`. You'd think a single `SecurityMiddleware` would do, but these four solve radically different problems. CORS is about browser-enforced cross-origin policy. Rate limiting is about resource protection at the network edge. CSRF is about token-based mutation verification. Security headers are about instructing the browser to lock itself down. They share nothing except an interface. That interface is the `Middleware` trait at `crates/ironic-http/src/pipeline.rs:30` — a single `handle` method that receives `&mut RequestContext` and a `MiddlewareNext`, returning a `PipelineFuture`. Every middleware below implements it, but what each does inside `handle` has nothing in common with the others.

---

## 1. CorsMiddleware — deny by default, allow explicitly

The `CorsMiddleware` at `crates/ironic-security/src/cors.rs:90` embodies the principle that your API should talk to nobody unless you say so. Its `CorsConfig` starts with an empty `allowed_origins` list (line 25). If you never call `allowed_origins(...)`, every cross-origin request is simply passed through without CORS headers — which means the browser will block it. That's the deny-by-default posture.

The core logic lives in `is_origin_allowed` at line 84: it returns `true` only if the origin string appears in the allowlist literally, or if `"*"` is present. There's no regex, no substring matching, no implicit scheme normalization. An origin is either in the list or it isn't.

Inside `Middleware::handle` (line 106), the middleware first reads the `Origin` header from the request. If there is no origin — meaning a same-origin request or a non-browser client — it passes straight through. Same if the origin isn't allowed: pass through, no headers, browser rejects it. This is intentional: CORS is a browser protocol, so non-browser traffic should never see CORS headers it doesn't need.

Preflight detection happens at lines 127–132: if the method is `OPTIONS` _and_ the `Access-Control-Request-Method` header is present, the middleware short-circuits with a `204 No Content` response. The `set_cors_headers` function (line 147) populates `Access-Control-Allow-Origin`, `Access-Control-Allow-Methods`, `Access-Control-Allow-Headers`, and `Access-Control-Max-Age`. When the allowlist contains `"*"` and `allow_credentials` is `false`, the origin header is set to the literal `"*"` — which means credentialed requests are never allowed with a wildcard, matching the spec. Otherwise the origin is reflected back and, if configured, `Access-Control-Allow-Credentials: true` is set.

For non-preflight requests, the middleware calls `next.run(context)` and then decorates the response with the same CORS headers. This means CORS headers are added to every response from an allowed origin, not just preflights.

---

## 2. RateLimitMiddleware — sliding windows, client IP isolation

`RateLimitMiddleware` at `crates/ironic-security/src/rate_limit.rs:87` wraps an `InMemoryRateLimiter` that uses a `Mutex<HashMap<String, Vec<WindowEntry>>>` (line 24). Each client key maps to a chronological list of timestamps. The algorithm is a textbook sliding window: on each request, prune all entries older than `window_secs`, count the survivors, and allow only if the count is below `max_requests`.

The `check` method at lines 45–61 acquires the mutex, retains only entries whose `duration_since` is within the window, and either appends a fresh entry (if allowed) or returns `false` (rate-limited). The `remaining` method at line 69 does the same retain logic read-only to compute remaining capacity — it calls `saturating_sub` at line 80 to avoid underflow.

Client isolation uses the client IP. The middleware reads the `x-forwarded-for` header at line 112, but the behavior is nuanced: it takes the _rightmost_ IP (line 114), which in a properly configured proxy chain is the one set by the first trusted proxy — the actual client. If the header is absent or empty, it falls back to `"127.0.0.1"`. This means running behind a reverse proxy requires it to set `x-forwarded-for`; otherwise all requests share the same bucket.

When the limit is hit (line 120), the middleware returns `429 Too Many Requests` with `Retry-After: 60` and `x-ratelimit-remaining: 0`. On success, `x-ratelimit-remaining` is set to the computed remaining count at line 139. The header name is constructed from a static string at line 131 — intentionally using `HeaderName::from_static` rather than the `http` crate's built-in constants, since `x-ratelimit-remaining` isn't a standard header.

---

## 3. CsrfMiddleware — synchronizer token pattern, cookie vs. header

`CsrfMiddleware` at `crates/ironic-security/src/csrf.rs:81` implements the classic synchronizer token pattern. The idea is simple: an attacker on a malicious site cannot read your cookies (the browser's same-origin policy prevents that), but your browser will send your cookies automatically on a cross-site POST. So we require the token to come from two places: a cookie (where the browser sends it automatically) and a custom header (where only your own JavaScript can set it). If they match, the request originated from your own page.

The configuration at lines 13–18 defines the cookie name (default `"csrf-token"`), the header name (default `"x-csrf-token"`), a token generator (default `uuid::Uuid::new_v4()`), and the safe methods list (`GET`, `HEAD`, `OPTIONS`). Both the cookie name and header name are validated at construction time — the cookie name must not contain `;`, `=`, `\r`, or `\n` (line 47); the header name must not contain `\r` or `\n` (line 66). These are panics, not runtime errors, because invalid names are a programmer bug.

The `Middleware::handle` at line 118 splits into two paths. For safe methods (line 128): if the CSRF cookie is already present, pass through. If not, generate a fresh token with `(self.config.token_generator)()` at line 131, call `next.run()` to get the response, then append a `Set-Cookie` header with `HttpOnly; Secure; SameSite=Strict` flags (line 136). The cookie is scoped to `/` by default.

For state-changing methods (POST, PUT, PATCH, DELETE, etc.): extract the cookie token via `extract_cookie_token` (line 94), which iterates the `Cookie` header, splits on `"; "`, and strips the configured cookie name prefix. Extract the header token via `extract_header_token` (line 108), which reads the configured header name directly. If both exist and are equal, the request proceeds. Otherwise, a `403 Forbidden` with error code `"RF_HTTP_CSRF_TOKEN_MISMATCH"` is returned (line 153). The comparison is a simple string equality — no hashing, no HMAC, no double-submit tricks.

---

## 4. SecurityHeadersMiddleware — nine headers, each individually configurable

`SecurityHeadersMiddleware` at `crates/ironic-security/src/security_headers.rs:125` is the most straightforward of the four. It takes a `SecurityHeadersConfig` where each of the nine supported headers is typed as `Option<String>`. A `Some` value means the header is set; a `None` value means it's omitted. Every header defaults to `Some` with a secure value (lines 26–34):

| Header | Default value |
|---|---|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` |
| `Content-Security-Policy` | `default-src 'self'` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `geolocation=()` |
| `Cross-Origin-Opener-Policy` | `same-origin` |
| `Cross-Origin-Embedder-Policy` | `require-corp` |
| `Cross-Origin-Resource-Policy` | `same-origin` |

Each header has a builder-style setter (e.g., `csp(…)`, `hsts(…)`, `x_frame_options(…)`) and a companion `disable_*` method that sets the field to `None`. Partial disabling is supported: you can turn off HSTS and keep CSP, or vice versa.

The `handle` method at line 140 is mechanical: run the next middleware/handler, then for each header field that is `Some`, call `insert_header`. The helper at line 117 calls `HeaderValue::from_str` and silently skips malformed values. There are no ifs, no conditions on request state, no short-circuiting — headers are appended unconditionally to every response that passes through.

---

## 5. One trait, four personalities

All four middlewares implement `Middleware` from `crates/ironic-http/src/pipeline.rs:30`, whose `handle` method signature is identical across all of them:

```rust
fn handle<'a>(
    &'a self,
    context: &'a mut RequestContext,
    next: MiddlewareNext<'a>,
) -> PipelineFuture<'a>;
```

But what each does inside `handle` reveals the trait's flexibility. `CorsMiddleware` inspects `context.request().headers()` to decide whether to short-circuit (preflight) or decorate the response. `RateLimitMiddleware` reads `x-forwarded-for` into a key, mutates shared state (the `Mutex<HashMap>`), and may short-circuit with a 429. `CsrfMiddleware` reads cookies and headers, conditionally sets `Set-Cookie`, and may short-circuit with a 403. `SecurityHeadersMiddleware` does exactly one thing: decorate the response after `next.run()` — it never short-circuits and never reads the request beyond what `next` provides.

This is the power of a single-trait middleware stack: each layer is a black box from the framework's perspective. The router doesn't know about CORS. The guard evaluator doesn't know about rate limits. But they compose. If you enable all four, a cross-origin POST triggers the following sequence: `CorsMiddleware` checks the origin → `RateLimitMiddleware` checks the IP bucket → `CsrfMiddleware` validates the token → `SecurityHeadersMiddleware` appends response headers. Four independent concerns, one stack, zero shared state.

---

The source for all four middlewares lives under `crates/ironic-security/src/`. They're feature-gated so you only pay for what you use: each compiles separately and can be omitted entirely at build time. But when enabled, they provide a defense-in-depth security baseline that requires no third-party crates — just the standard library, the `http` crate, and `uuid` for CSRF token generation.
