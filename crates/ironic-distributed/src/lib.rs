//! Optional messaging and application-architecture integrations.
//!
//! Sub-modules are feature-gated:
//! - `cqrs` — command/query responsibility segregation
//! - `graphql` — GraphQL schema integration (requires `async-graphql`)
//! - `grpc` — gRPC integration (requires `tonic`)
//! - `microservices` — channel-based microservice transport
//! - `queues` — at-least-once queue abstraction with in-memory implementation
//! - `sagas` — ordered saga execution with reverse compensation

#[cfg(feature = "cqrs")]
pub mod cqrs;
#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "microservices")]
pub mod microservices;
#[cfg(feature = "queues")]
pub mod queues;
#[cfg(feature = "sagas")]
pub mod sagas;
