---
title: Cookie & Raw Body
description: Extract cookies and raw request bodies from incoming requests
---

# Cookie & Raw Body

## Cookie Extractor

Extract a named cookie value from the request using `#[cookie]`:

```rust
#[get("/profile")]
async fn profile(#[cookie("session_id")] session_id: String) -> String {
    format!("your session: {session_id}")
}
```

The `CookieParameter<String>` extractor parses the `Cookie` header and
returns the value for the specified cookie name. Returns `400 Bad Request`
if the cookie is absent.

## Raw Body Extractor

Access the raw request body bytes using `#[raw_body]`:

```rust
#[post("/upload")]
async fn upload(#[raw_body] body: Vec<u8>) -> String {
    format!("received {} bytes", body.len())
}
```

The `RawBody` extractor returns the request body as `Vec<u8>`, useful for
binary payloads, file uploads, or custom parsing.
