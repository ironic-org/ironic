---
title: Content Negotiation
description: Read the Accept header to serve JSON, XML, or any format your client prefers.
---

# Content Negotiation

## What is it?

Clients tell the server what format they want via the `Accept` header:

```
GET /api/users HTTP/1.1
Accept: application/xml
```

The server should respond accordingly. `RequestContext` provides helpers to read this header.

## How to use

```rust
use ironic::prelude::*;

#[get("/users")]
async fn list(&self, context: &mut RequestContext) -> Result<FrameworkResponse, HttpError> {
    match context.preferred_content_type() {
        Some("application/xml") => self.render_xml(),
        _ => self.render_json(),
    }
}
```

## API

| Method | Returns | Description |
|--------|---------|-------------|
| `preferred_content_type()` | `Option<&str>` | Highest-weighted MIME type from `Accept` header |
| `accepts_json()` | `bool` | `true` if client prefers JSON or `*/*` |

## Quick check

```rust
if context.accepts_json() {
    FrameworkResponse::json(HttpStatus::OK, &data)
} else {
    FrameworkResponse::bytes(HttpStatus::OK, xml_data)
}
```

## How it parses

The `Accept` header is comma-separated with optional quality values:

```
text/html, application/xhtml+xml, application/xml;q=0.9, */*;q=0.8
```

`preferred_content_type()` returns the first type before any `;` or `,` — `"text/html"` in this case. Quality weights are not fully parsed yet (future enhancement).
