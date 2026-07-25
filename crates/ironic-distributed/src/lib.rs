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
#[cfg(feature = "microservices")]
pub mod tracing;
#[cfg(feature = "transport-kafka")]
pub mod transport_kafka;
#[cfg(feature = "transport-mqtt")]
pub mod transport_mqtt;
#[cfg(feature = "transport-nats")]
pub mod transport_nats;
#[cfg(feature = "transport-rabbitmq")]
pub mod transport_rabbitmq;
#[cfg(feature = "transport-redis")]
pub mod transport_redis;
#[cfg(feature = "microservices")]
pub mod transport_tcp;
