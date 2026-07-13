use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use ironic_di::{ProviderKey, ProviderValue};
use ironic_platform::ShutdownSignal;

/// The asynchronous result of a lifecycle callback.
pub type LifecycleFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), LifecycleError>> + Send + 'a>>;

pub(crate) type InitCallback = Arc<dyn Fn(ProviderValue) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ShutdownCallback =
    Arc<dyn Fn(ProviderValue, ShutdownSignal) -> LifecycleFuture<'static> + Send + Sync>;

/// A safe lifecycle callback failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct LifecycleError {
    message: String,
}

impl LifecycleError {
    /// Creates a lifecycle error with a safe diagnostic message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Runs after a provider's module and dependencies are available.
pub trait OnModuleInit: Send + Sync + 'static {
    /// Initializes the provider.
    fn on_module_init(&self) -> LifecycleFuture<'_>;
}

/// Runs after every module initialization callback has succeeded.
pub trait OnApplicationBootstrap: Send + Sync + 'static {
    /// Completes application-level startup work.
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;
}

/// Runs during cleanup in reverse successful-initialization order.
pub trait OnModuleDestroy: Send + Sync + 'static {
    /// Releases module-owned resources.
    fn on_module_destroy(&self) -> LifecycleFuture<'_>;
}

/// Runs after serving stops and before module destruction.
pub trait OnApplicationShutdown: Send + Sync + 'static {
    /// Handles the shutdown signal.
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}

/// Type-erased lifecycle callbacks registered for one provider.
#[derive(Clone)]
pub struct LifecycleDefinition {
    key: ProviderKey,
    pub(crate) module_init: Option<InitCallback>,
    pub(crate) application_bootstrap: Option<InitCallback>,
    pub(crate) module_destroy: Option<InitCallback>,
    pub(crate) application_shutdown: Option<ShutdownCallback>,
}

impl LifecycleDefinition {
    /// Starts a lifecycle definition for provider `T`.
    #[must_use]
    pub fn builder<T: Send + Sync + 'static>() -> LifecycleDefinitionBuilder<T> {
        LifecycleDefinitionBuilder {
            definition: Self {
                key: ProviderKey::of::<T>(),
                module_init: None,
                application_bootstrap: None,
                module_destroy: None,
                application_shutdown: None,
            },
            marker: PhantomData,
        }
    }

    /// Returns the lifecycle provider key.
    #[must_use]
    pub const fn key(&self) -> ProviderKey {
        self.key
    }
}

impl std::fmt::Debug for LifecycleDefinition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LifecycleDefinition")
            .field("key", &self.key)
            .field("module_init", &self.module_init.is_some())
            .field(
                "application_bootstrap",
                &self.application_bootstrap.is_some(),
            )
            .field("module_destroy", &self.module_destroy.is_some())
            .field("application_shutdown", &self.application_shutdown.is_some())
            .finish()
    }
}

/// Builds explicit lifecycle metadata for provider `T`.
pub struct LifecycleDefinitionBuilder<T> {
    definition: LifecycleDefinition,
    marker: PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> LifecycleDefinitionBuilder<T> {
    /// Registers [`OnModuleInit`].
    #[must_use]
    pub fn module_init(mut self) -> Self
    where
        T: OnModuleInit,
    {
        self.definition.module_init = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_module_init().await
            })
        }));
        self
    }

    /// Registers [`OnApplicationBootstrap`].
    #[must_use]
    pub fn application_bootstrap(mut self) -> Self
    where
        T: OnApplicationBootstrap,
    {
        self.definition.application_bootstrap = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_application_bootstrap().await
            })
        }));
        self
    }

    /// Registers [`OnModuleDestroy`].
    #[must_use]
    pub fn module_destroy(mut self) -> Self
    where
        T: OnModuleDestroy,
    {
        self.definition.module_destroy = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_module_destroy().await
            })
        }));
        self
    }

    /// Registers [`OnApplicationShutdown`].
    #[must_use]
    pub fn application_shutdown(mut self) -> Self
    where
        T: OnApplicationShutdown,
    {
        self.definition.application_shutdown = Some(Arc::new(|value, signal| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_application_shutdown(signal).await
            })
        }));
        self
    }

    /// Completes the lifecycle definition.
    #[must_use]
    pub fn build(self) -> LifecycleDefinition {
        self.definition
    }
}

fn downcast<T: Send + Sync + 'static>(value: ProviderValue) -> Result<Arc<T>, LifecycleError> {
    value.downcast::<T>().map_err(|_| {
        LifecycleError::new(format!(
            "Lifecycle provider type mismatch for `{}`",
            std::any::type_name::<T>()
        ))
    })
}
