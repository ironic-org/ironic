#![doc = "Platform adapter contracts for Ironic."]

use std::{error::Error, future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use ironic_http::CompiledHttpApplication;

/// The reason application serving stopped.
///
/// # Examples
///
/// ```rust
/// use ironic::ShutdownSignal;
///
/// let signal = ShutdownSignal::Interrupt;
/// assert_eq!(signal, ShutdownSignal::Interrupt);
/// ```
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
///
/// # Examples
///
/// ```rust
/// use ironic::{Shutdown, ShutdownSignal};
///
/// # async fn example() {
/// let shutdown = Shutdown::new(async { ShutdownSignal::Interrupt });
/// let signal = shutdown.wait().await;
/// assert_eq!(signal, ShutdownSignal::Interrupt);
/// # }
/// ```
pub struct Shutdown {
    future: Pin<Box<dyn Future<Output = ShutdownSignal> + Send + 'static>>,
}

impl Shutdown {
    /// Creates a shutdown trigger from an owned future.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ironic::{Shutdown, ShutdownSignal};
    ///
    /// let shutdown = Shutdown::new(std::future::ready(ShutdownSignal::Terminate));
    /// ```
    #[must_use]
    pub fn new(future: impl Future<Output = ShutdownSignal> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    /// Waits for the shutdown signal.
    ///
    /// Consumes the trigger and returns the signal that caused shutdown.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ironic::{Shutdown, ShutdownSignal};
    ///
    /// # async fn example() {
    /// let shutdown = Shutdown::new(async { ShutdownSignal::Interrupt });
    /// let signal = shutdown.wait().await;
    /// # }
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_signal_equality() {
        assert_eq!(ShutdownSignal::Interrupt, ShutdownSignal::Interrupt);
        assert_eq!(ShutdownSignal::Terminate, ShutdownSignal::Terminate);
        assert_eq!(ShutdownSignal::Custom("a"), ShutdownSignal::Custom("a"));
        assert_ne!(ShutdownSignal::Interrupt, ShutdownSignal::Terminate);
        assert_ne!(ShutdownSignal::Custom("a"), ShutdownSignal::Custom("b"));
    }

    #[test]
    fn shutdown_signal_debug() {
        assert_eq!(format!("{:?}", ShutdownSignal::Interrupt), "Interrupt");
        assert_eq!(format!("{:?}", ShutdownSignal::Terminate), "Terminate");
        assert_eq!(
            format!("{:?}", ShutdownSignal::Custom("db")),
            "Custom(\"db\")"
        );
    }

    #[test]
    fn shutdown_signal_clone_and_copy() {
        let a = ShutdownSignal::Interrupt;
        let b = a;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn shutdown_wait_returns_signal() {
        let shutdown = Shutdown::new(std::future::ready(ShutdownSignal::Interrupt));
        let signal = shutdown.wait().await;
        assert_eq!(signal, ShutdownSignal::Interrupt);
    }

    #[tokio::test]
    async fn shutdown_custom_signal() {
        let shutdown = Shutdown::new(std::future::ready(ShutdownSignal::Custom("migration")));
        let signal = shutdown.wait().await;
        assert_eq!(signal, ShutdownSignal::Custom("migration"));
    }

    #[tokio::test]
    async fn shutdown_terminate_signal() {
        let shutdown = Shutdown::new(std::future::ready(ShutdownSignal::Terminate));
        let signal = shutdown.wait().await;
        assert_eq!(signal, ShutdownSignal::Terminate);
    }
}
