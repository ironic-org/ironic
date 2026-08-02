---
title: HTTP Client
description: Make inter-service HTTP requests with HttpClientService
---

# HTTP Client

The `HttpClientService` provides an injectable HTTP client for inter-service
communication with retry and circuit breaker support.

## Setup

```toml
[dependencies]
ironic = { features = ["http-client"] }
```

## Usage

```rust
use ironic::services::http_client::HttpClientService;

#[derive(Injectable)]
struct OrderService {
    http: HttpClientService,
}

impl OrderService {
    async fn get_user(&self, id: &str) -> Result<User, HttpClientError> {
        self.http.get(&format!("http://users/{id}")).await
    }
}
```

## With Retry

```rust
let user: User = self.http
    .with_retry(3, 100)     // 3 retries, 100ms base delay
    .get("http://users/1")
    .await?;
```

## With Circuit Breaker

```rust
let user: User = self.http
    .with_circuit_breaker("user-svc")
    .get("http://users/1")
    .await?;
```

## Methods

| Method | HTTP Verb | Description |
|--------|-----------|-------------|
| `get::<T>(url)` | GET | Fetch and deserialize JSON |
| `post::<T, R>(url, body)` | POST | Send JSON, receive JSON |
| `put::<T, R>(url, body)` | PUT | Update resource |
| `delete(url)` | DELETE | Remove resource |

## Error Handling

```rust
match result {
    Ok(value) => ...,
    Err(HttpClientError::Request(e)) => // network error
    Err(HttpClientError::CircuitOpen) => // circuit breaker open
    Err(HttpClientError::Deserialize(e)) => // JSON parse error
    Err(HttpClientError::Internal(e)) => // task spawn error
}
```

## Distributed Tracing

All requests automatically include the `traceparent` header from the
current tracing span, enabling end-to-end trace propagation.
