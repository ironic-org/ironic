#![doc = "Platform adapter contracts for Ironic."]

use std::{error::Error, future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use ironic_http::CompiledHttpApplication;

/// The reason application serving stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShutdownSignal {
    /// The process received an interrupt signal.
    Interrupt,
    /// The process received a termination signal.
    Terminate,
    /// A user-provided shutdown future completed.
    Custom(&'static str),
}

/// An owned asynchronous shutdown trigger.
pub struct Shutdown {
    future: Pin<Box<dyn Future<Output = ShutdownSignal> + Send + 'static>>,
}

impl Shutdown {
    /// Creates a shutdown trigger from an owned future.
    #[must_use]
    pub fn new(future: impl Future<Output = ShutdownSignal> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    /// Waits for the shutdown signal.
    pub async fn wait(self) -> ShutdownSignal {
        self.future.await
    }
}

/// The asynchronous result of platform serving.
pub type PlatformFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Builds a native HTTP application from transport-neutral runtime state.
pub trait HttpPlatformAdapter: Send + 'static {
    /// The native application returned after route compilation.
    type Application: HttpPlatformApplication<Error = Self::Error>;
    /// A platform-specific startup or serving error.
    type Error: Error + Send + Sync + 'static;

    /// Builds the native application without performing network I/O.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when native route or application construction fails.
    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error>;
}

/// Serves a built native HTTP application.
pub trait HttpPlatformApplication: Send + 'static {
    /// The platform-specific serving error.
    type Error: Error + Send + Sync + 'static;

    /// Binds `address`, serves requests, and stops gracefully after `shutdown` completes.
    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>>;
}
