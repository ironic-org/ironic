---
title: "Field-level JSON Redaction — how SerializeInterceptor guards sensitive data"
description: "A deep dive into Ironic's interceptor-based JSON field redaction: how it walks the serde_json Value tree, applies dotted-path rules, respects user roles, and keeps offensive data off the wire."
date: "2026-07-15"
author: "Ironic Team"
---

# Field-level JSON Redaction — how SerializeInterceptor guards sensitive data

APIs leak data. It happens in every codebase: a developer adds an `ssn` field to the user DTO, a `credit_card` to the order response, or an `internal_token` to the provider response — and suddenly the framework is serializing secrets into every JSON response that goes over the wire. The fix is usually either over-fetch prevention (GraphQL-style) or ad-hoc per-endpoint filtering. Both approaches are error-prone and easy to miss during code review.

Ironic takes a different approach. The `SerializeInterceptor` runs in the response pipeline — after your handler executes, before bytes hit the socket — and prunes your JSON payloads according to declarative rules. You declare what to hide once, and the framework enforces it on every response. No manual filtering, no forgotten endpoints.

---

## Where it sits in the pipeline

The interceptor chain is the final processing layer before a response leaves the framework. In `crates/ironic-http/src/serialization.rs:108`, `SerializeInterceptor` implements the `Interceptor` trait, which means it wraps handler execution. When the handler returns a response, the interceptor gets a mutable reference to it:

```rust
fn intercept(&self, context: &mut RequestContext, next: InterceptorNext) -> PipelineFuture {
    Box::pin(async move {
        let mut response = next.run(context).await?;
        // ... inspect, transform, return
    })
}
```

The key insight: by the time the interceptor sees the response, guards have run, authentication has been checked, and the handler has produced its output. The interceptor is purely a post-processing step. It can't prevent the handler from running, but it can prevent the handler's output from reaching the client intact.

---

## JSON-only by contract

Not every response is JSON. A file download, an HTML template, or a redirect doesn't need field redaction. The interceptor inspects the `Content-Type` header (`application/json`) and short-circuits immediately if it's anything else:

```rust
let is_json = response.headers()
    .get(http::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .is_some_and(|v| v.starts_with("application/json"));

if !is_json {
    return Ok(response);
}
```

This is important: it means the interceptor has zero overhead for non-JSON responses. The header check is a single string prefix match.

---

## Deserialize, walk, strip, re-serialize

Once the interceptor confirms a JSON response, it deserializes the raw bytes into a `serde_json::Value` tree:

```rust
let mut value: Value = serde_json::from_slice(&body).map_err(|e| {
    HttpError::internal(
        "RF_SERIALIZE_INTERCEPTOR_PARSE_FAILED",
        format!("Failed to parse JSON response: {e}"),
    )
})?;
```

Then it extracts the current user's roles from the request context, applies the field rules in-place on the `Value` tree, and re-serializes the pruned tree back into bytes:

```rust
apply_rules(&mut value, &self.rules, &roles);

let new_body = serde_json::to_vec(&value).map_err(|e| {
    HttpError::internal(
        "RF_SERIALIZE_INTERCEPTOR_SERIALIZE_FAILED",
        format!("Failed to re-serialize JSON response: {e}"),
    )
})?;
```

The full cycle is `parse → walk → strip → serialize`. There's no partial streaming approach — this is a complete transformation of the response body.

---

## Dotted-path field traversal

The field rules use a dot-separated path syntax. `"user.ssn"` means "the `ssn` field inside the `user` object." The `apply_rules` function at line 158 handles this by splitting on the first dot:

```rust
if let Some((top, _rest)) = field_path.split_once('.') {
    if let Some(nested) = map.get_mut(top) {
        let sub_rules = {
            let mut r = FieldRules::new();
            for (fp, rl) in &rules.rules {
                if let Some(suffix) = fp.strip_prefix(&format!("{top}.")) {
                    r.rules.push((suffix.to_string(), rl.clone()));
                }
            }
            r
        };
        apply_rules(nested, &sub_rules, roles);
    }
}
```

When a rule targets `"user.ssn"`, the function delegates to the nested `user` object with a trimmed subset of rules. This means a single rule pass handles arbitrarily deep nesting — `"a.b.c.d"` recurses three times, each time peeling off the next segment and narrowing the rule set.

---

## Two rule types: Exclude and Expose

`FieldRule` is an enum with two variants:

```rust
pub enum FieldRule {
    Exclude,
    Expose { role: String },
}
```

**`Exclude`** is unconditional: the field is removed regardless of who's asking. This is for secrets, internal identifiers, and anything that should never leave the server.

**`Expose { role }`** is conditional: the field is removed unless the current user has the specified role. The check is a simple list membership test:

```rust
FieldRule::Expose { role } => {
    if !roles.iter().any(|r| r == role) {
        to_remove.push(field_path.clone());
    }
}
```

This enables patterns like "admins can see the full audit log, everyone else gets a sanitized version" — all from a single rule declaration on the route.

---

## Role propagation via RequestContext

How does the interceptor know who the user is? The `CurrentRoles` extension is set on the request context by an auth middleware or guard earlier in the pipeline:

```rust
pub fn set_current_roles(context: &mut RequestContext, roles: Vec<String>) {
    context.insert_extension(CurrentRoles(roles));
}
```

The interceptor reads them back through a private helper:

```rust
fn current_roles(context: &RequestContext) -> Vec<String> {
    context.extension::<CurrentRoles>()
        .map(|r| r.0.clone())
        .unwrap_or_default()
}
```

If no auth middleware has run, the roles list is empty — and every `Expose` field will be stripped. This is safe-by-default: anonymous requests never see role-gated fields.

---

## Recursive array handling

The rule engine doesn't treat arrays specially. When it encounters a `Value::Array`, it iterates over every element and applies the same rules recursively:

```rust
Value::Array(arr) => {
    for item in arr.iter_mut() {
        apply_rules(item, rules, roles);
    }
}
```

This means a rule like `.exclude("token")` will strip the `token` field from every object in a JSON array response. If your handler returns a list of 10,000 users, each one gets the same redaction — automatically.

---

## The cost

The full parse-transform-serialize cycle is not free. Every JSON response that passes through `SerializeInterceptor` pays the cost of:

1. **Deserialization** — `serde_json::from_slice` allocates a full `Value` tree
2. **Tree traversal** — `apply_rules` walks every node, even if there are no rules matching that subtree
3. **Re-serialization** — `serde_json::to_vec` allocates a new `Vec<u8>`

For small responses under 1 KB with a handful of rules, this is negligible. For large responses (megabytes of JSON with deeply nested structures) it can be measurable. The trade-off is deliberate: security and correctness over throughput.

If you're serving public JSON feeds at scale and don't need field-level redaction, skip the interceptor. If you're serving user data with PII, the overhead is a fraction of what it would cost to implement the same guarantees by hand — and unlike manual filtering, it can't be accidentally skipped in one branch of a handler.
