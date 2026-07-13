---
title: API versioning
description: Version controllers with URI prefix, Accept-Version header, or media type strategies.
---

# API versioning

Enable `versioning` to attach version metadata to controllers. Ironic supports three versioning
strategies that route requests to the correct controller at runtime.

```toml
ironic = { features = ["versioning"] }
```

## Strategies

| Strategy | How it works | Example |
|----------|-------------|---------|
| `Uri` | Version prefix in the URI path | `/v1/users` → Controller v1 |
| `Header` | `Accept-Version` header on the request | `Accept-Version: 2024-01-01` |
| `MediaType` | Version parameter in `Accept` header | `Accept: application/vnd.api+json;version=1` |

## Declaring versions

```rust
use ironic::{controller, VersionMetadata, VersioningStrategy};

#[controller("/users", version = "1", strategy = "uri")]
struct UsersV1;

#[routes]
impl UsersV1 {
    #[get("/")]
    async fn list(&self) -> Result<impl IntoFrameworkResponse, HttpError> {
        // Handle v1 listing
    }
}

#[controller("/users", version = "2", strategy = "uri")]
struct UsersV2;

#[routes]
impl UsersV2 {
    #[get("/")]
    async fn list(&self) -> Result<impl IntoFrameworkResponse, HttpError> {
        // Handle v2 listing with a different response shape
    }
}
```

## Header-based versioning

```rust
#[controller("/reports", version = "2024-01-01", strategy = "header")]
struct ReportsController;
```

When the client sends `Accept-Version: 2024-01-01`, requests are routed to this controller.
Requests without the header or with an unmatched version receive `404 Not Found`.

## Chaining multiple versions

Register multiple versions of the same path prefix and the platform adapter resolves the best
match at dispatch time. Routes from all registered versions are compiled into a single route
table with the version strategy applied.

## `VersionMetadata`

The metadata type is reusable beyond controllers. Attach it to individual routes when a
controller exposes routes at different versions:

```rust
use ironic::{VersionMetadata, VersioningStrategy, RouteMetadata};

route.metadata(VersionMetadata::new("2", VersioningStrategy::Uri));
```
