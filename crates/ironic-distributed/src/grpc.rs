//! gRPC integration through the upstream Tonic runtime.

/// The upstream Tonic API used for generated services, clients, status, and transport.
pub use ::tonic as driver;

use crate::ProviderDefinition;

/// Registers a reusable Tonic channel as a singleton provider.
#[must_use]
pub fn channel_provider(channel: driver::transport::Channel) -> ProviderDefinition {
    ProviderDefinition::value(channel)
}
