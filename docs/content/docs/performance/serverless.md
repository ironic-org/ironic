---
title: Serverless
description: Deploy Ironic applications on AWS Lambda
---

# Serverless (AWS Lambda)

Ironic applications can run on AWS Lambda using the `serverless` feature.

## Setup

```toml
[dependencies]
ironic = { features = ["serverless"] }
```

## Usage

After building the application, call `run_lambda()` instead of `listen()`:

```rust
use ironic::prelude::*;

#[ironic::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::builder()
        .module(AppModule)
        .platform(AxumAdapter::new())
        .build()
        .await?;

    // Run as Lambda (blocks forever)
    app.platform.into_application().run_lambda().await?;
    Ok(())
}
```

## How It Works

The `AxumApplication::run_lambda()` method wraps the compiled Axum router
with the `lambda_http` runtime, which processes API Gateway events and
converts them to HTTP requests.

## Limitations

- WebSocket connections are not supported in Lambda
- Long-lived connections (SSE) have a 29-second timeout
- File uploads are limited by Lambda's 6MB payload size
