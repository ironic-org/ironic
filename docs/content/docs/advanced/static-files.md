---
title: Static File Serving
description: Serve static assets — images, CSS, JavaScript — with caching and ETag support.
---

# Static File Serving

## What you'll learn

- Serve static files from a directory with a single method call
- Configure caching with `Cache-Control` headers
- Automatic ETag generation for conditional requests (304 responses)
- Real-world patterns: versioned assets, SPAs with fallback, multiple directories
- How caching headers affect browser behaviour

---

## Enabling static files

```toml
ironic = { features = ["static-files"] }
```

## Basic usage

```rust
use ironic::AxumAdapter;

let adapter = AxumAdapter::new()
    .static_files("/static", "public/");
```

This serves every file under the `public/` directory at the `/static/*` path
prefix.  For example:

| File on disk | URL |
|---|---|
| `public/logo.png` | `GET /static/logo.png` |
| `public/css/app.css` | `GET /static/css/app.css` |
| `public/js/bundle.js` | `GET /static/js/bundle.js` |

## Real-world patterns

### Pattern 1: Single-page application (SPA) fallback

```rust
let adapter = AxumAdapter::new()
    .static_files("/", "dist/");  // SPA build output
```

> **Caveat:** This registers a catch-all route that may conflict with your API
> routes.  Serve the SPA from a subdomain or register static files **after**
> all API routes using the platform adapter's router ordering.

### Pattern 2: Separate directories for different asset types

```rust
let adapter = AxumAdapter::new()
    .static_files("/images", "data/images/")
    .static_files("/assets", "dist/assets/")
    .static_files("/favicon", "data/icons/");
```

Each call adds an independent directory-to-route mapping.

### Pattern 3: Versioned assets with long cache

```rust
use ironic::StaticFileConfig;

let adapter = AxumAdapter::new()
    .static_files("/static", "public/")
    .static_files_with_opts("/build", "dist/", StaticFileConfig {
        cache_control: "public, max-age=31536000, immutable".into(),  // 1 year
    });
```

Use content-hashed filenames (`app.a1b2c3.js`) with `immutable` caching so
browsers never re-validate.  When the file changes, the hash changes and the
browser fetches the new URL.

## Configuration

```rust
StaticFileConfig {
    cache_control: "public, max-age=604800".into(),  // 7 days
}
```

| Option | Default | Description |
|--------|---------|-------------|
| `cache_control` | `"public, max-age=3600"` | `Cache-Control` header value on every response |

### Cache-Control strategy guide

| Value | Use case |
|-------|----------|
| `public, max-age=0, must-revalidate` | Always fresh — good for HTML pages |
| `public, max-age=3600` | One hour — good for JS/CSS that changes frequently |
| `public, max-age=604800` | One week — good for stable assets |
| `public, max-age=31536000, immutable` | One year + immutable — content-hashed files only |

## How ETags and 304 work

Ironic uses `tower-http::services::ServeDir` under the hood, which generates
`ETag` headers from file metadata (inode, mtime, size).

```
Request:
  GET /static/logo.png
  If-None-Match: "e4a5b6c7-1a2b-3c4d"

Response (if unchanged):
  HTTP/1.1 304 Not Modified
  ETag: "e4a5b6c7-1a2b-3c4d"

Response (if changed):
  HTTP/1.1 200 OK
  ETag: "f8g9h0i1-2j3k-4l5m"
  Content-Type: image/png
  Cache-Control: public, max-age=3600
```

This saves bandwidth: the browser sends a lightweight conditional GET instead of
downloading the full file.  ETags work even when the file content changes at the
same path, unlike time-based caching alone.

## Testing static file serving

```rust
#[cfg(test)]
mod tests {
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_serves_existing_file() {
        let app = build_app();
        let response = app
            .oneshot(Request::get("/static/test.txt").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_returns_404_for_missing_file() {
        let app = build_app();
        let response = app
            .oneshot(Request::get("/static/nonexistent.txt").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_etag_returns_304() {
        let app = build_app();
        // First request gets the file + ETag
        let response = app
            .clone()
            .oneshot(Request::get("/static/test.txt").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        let etag = response.headers().get("etag").unwrap().to_str().unwrap().to_owned();

        // Second request with If-None-Match returns 304
        let response = app
            .oneshot(
                Request::get("/static/test.txt")
                    .header("if-none-match", &etag)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }
}
```

## Security considerations

1. **Directory traversal:** `tower-http` prevents `../` attacks.  A request for
   `/static/../../etc/passwd` returns 404.
2. **Sensitive files:** Do not place `.env`, config files, or private keys in
   the served directory.
3. **CORS:** Static file serving does not add CORS headers.  If assets are
   served to a different origin, add a CORS middleware.
4. **Symlinks:** `ServeDir` follows symlinks.  Ensure symlinks in the served
   directory don't point to sensitive locations.

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Static files not served | Add `ironic = { features = ["static-files"] }` to `Cargo.toml` |
| Missing directory at startup | Verify the directory exists — the adapter does not create it |
| Cache stale assets in production | Use content-hashed filenames and `immutable` cache directive |
| SPA routing broken on refresh | Configure the platform to fall back to `index.html` for non-API routes |
| Sensitive files exposed | Never place secrets or config in the public directory |
| No CORS for cross-origin assets | Add `CorsMiddleware` if the frontend is served from a different origin |

## What you learned

- [x] `static_files(route, dir)` maps a directory to a URL path prefix
- [x] `StaticFileConfig.cache_control` sets the `Cache-Control` header
- [x] ETags are auto-generated from file metadata; `304 Not Modified` responses save bandwidth
- [x] Multiple `static_files()` calls compose for different asset directories
- [x] Immutable caching with content-hashed filenames maximizes cache efficiency
- [x] Directory traversal is automatically prevented
