//! Optional messaging and application-architecture integrations.

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
