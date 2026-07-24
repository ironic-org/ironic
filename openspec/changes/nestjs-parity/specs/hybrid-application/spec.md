## ADDED Requirements

### Requirement: Hybrid application mode
The framework SHALL allow a single Ironic application to serve HTTP requests AND act as a microservice client/server simultaneously.

#### Scenario: HTTP + microservice server
- **WHEN** `Application::builder().module(AppModule).microservice_server(RedisServer::new(cfg)).build()`
- **THEN** the app starts both the HTTP server and the Redis microservice listener, with lifecycle hooks managing both

#### Scenario: HTTP + microservice client
- **WHEN** a controller injects `MicroserviceClient` and calls `client.send()`
- **THEN** the HTTP handler communicates with remote services via the configured transport

### Requirement: Lifecycle integration
Microservice lifecycle SHALL integrate with Ironic's existing lifecycle hooks (OnModuleInit, OnServerReady, BeforeShutdown, etc.).

#### Scenario: Startup ordering
- **WHEN** the application starts
- **THEN** microservice clients connect first, then servers listen, then HTTP server starts

#### Scenario: Graceful shutdown
- **WHEN** the application receives a shutdown signal
- **THEN** the microservice servers close first, then clients disconnect, then HTTP server drains
