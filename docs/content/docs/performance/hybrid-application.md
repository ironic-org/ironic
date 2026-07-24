---
title: Hybrid Application
description: Run HTTP and microservice servers in the same process
---

# Hybrid Application

Ironic supports running an HTTP server and microservice endpoints in the
same process — a pattern known as a *hybrid application*.

## Usage

Use `.microservice_server()` and `.microservice_client()` on the application
builder:

```rust
Application::builder()
    .module(AppModule)
    .microservice_server(RedisServer::new(config))
    .microservice_client(RedisClient::new(config))
    .platform(AxumAdapter::new())
    .build()
    .await?
    .listen("0.0.0.0:3000")
    .await?;
```

## Lifecycle

Microservice servers are started after the DI container initializes but before
the HTTP server begins accepting requests. During shutdown, microservice
servers are closed first, then the HTTP server drains.

## Custom Transport Strategy

Register a paired client and server together:

```rust
struct MyTransport;

impl CustomTransportStrategy for MyTransport {
    type Client = MyClient;
    type Server = MyServer;
    fn create(self) -> (MyClient, MyServer) {
        (MyClient::new(), MyServer::new())
    }
}

Application::builder()
    .custom_transport(MyTransport)
    .build()
    .await?;
```
