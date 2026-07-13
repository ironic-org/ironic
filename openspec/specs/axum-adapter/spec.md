# Axum Adapter

## Purpose

Platform adapter converting compiled routes into an Axum router with DI-resolved controllers, Tower layers, and escape hatches.

## Requirements

### Requirement: Axum adapter SHALL convert framework routes to Axum Router
The Axum platform adapter SHALL convert compiled route definitions into an Axum Router, resolving controller instances through DI.

#### Scenario: Route is registered on Axum router
- **WHEN** a compiled route exists for `GET /users/:id`
- **THEN** the Axum adapter SHALL register a corresponding route on the Axum router that invokes the controller handler

### Requirement: Axum adapter SHALL support Tower layers
The Axum adapter SHALL allow registering Tower middleware layers that wrap all framework routes.

#### Scenario: Tower layer is applied
- **WHEN** a Tower layer is registered on the adapter
- **THEN** the layer SHALL wrap all routes on the Axum router

### Requirement: Axum adapter SHALL expose raw Axum router escape hatch
The framework SHALL provide an escape hatch to access the raw Axum Router for custom routes or middleware.

#### Scenario: Raw Axum router is accessible
- **WHEN** a user needs to add a custom Axum route
- **THEN** they SHALL be able to access and modify the underlying Axum Router

### Requirement: Axum adapter SHALL support graceful shutdown
The adapter SHALL listen for shutdown signals and perform a graceful shutdown, running lifecycle destroy hooks.

#### Scenario: SIGTERM triggers graceful shutdown
- **WHEN** the application receives SIGTERM
- **THEN** the adapter SHALL stop accepting new requests and run shutdown hooks
