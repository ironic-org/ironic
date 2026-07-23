//! gRPC integration through the upstream Tonic runtime.

/// The upstream Tonic API used for generated services, clients, status, and transport.
pub use ::tonic as driver;

use crate::ProviderDefinition;

/// Registers a reusable Tonic channel as a singleton provider.
///
/// # Examples
///
/// ```ignore
/// use ironic::distributed::grpc::channel_provider;
/// use tonic::transport::Channel;
///
/// let channel = Channel::from_static("http://localhost:50051");
/// let provider = channel_provider(channel);
/// ```
#[must_use]
pub fn channel_provider(channel: driver::transport::Channel) -> ProviderDefinition {
    ProviderDefinition::value(channel)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tonic_driver_re_exports() {
        let _ = super::driver::Code::Ok;
        let _ = super::driver::Status::new(super::driver::Code::Ok, "test");
    }

    #[test]
    fn channel_provider_type_check() {
        fn _assert(_c: tonic::transport::Channel) {}
    }
}
