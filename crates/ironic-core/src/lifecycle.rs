use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use ironic_di::{ProviderKey, ProviderValue};
use ironic_platform::ShutdownSignal;

/// The asynchronous result of a lifecycle callback.
pub type LifecycleFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), LifecycleError>> + Send + 'a>>;

pub(crate) type InitCallback = Arc<dyn Fn(ProviderValue) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ShutdownCallback =
    Arc<dyn Fn(ProviderValue, ShutdownSignal) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ConfigureCallback =
    Arc<dyn Fn(ProviderValue, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ServerReadyCallback =
    Arc<dyn Fn(ProviderValue) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type RequestInitCallback =
    Arc<dyn Fn(ProviderValue, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type RequestDestroyCallback =
    Arc<dyn Fn(ProviderValue) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ErrorCallback =
    Arc<dyn Fn(ProviderValue, String, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type GuardDeniedCallback =
    Arc<dyn Fn(ProviderValue, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type BeforeShutdownCallback =
    Arc<dyn Fn(ProviderValue, ShutdownSignal) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type AfterShutdownCallback =
    Arc<dyn Fn(ProviderValue) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ModuleLoadCallback =
    Arc<dyn Fn(ProviderValue, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type ModuleUnloadCallback =
    Arc<dyn Fn(ProviderValue, String) -> LifecycleFuture<'static> + Send + Sync>;
pub(crate) type AsyncInitCallback = Arc<
    dyn Fn(ProviderValue, std::sync::Arc<ironic_di::Container>) -> LifecycleFuture<'static>
        + Send
        + Sync,
>;

/// A safe lifecycle callback failure.
///
/// # Examples
///
/// ```rust
/// use ironic::LifecycleError;
///
/// let err = LifecycleError::new("db connection failed");
/// assert_eq!(err.to_string(), "db connection failed");
/// ```
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct LifecycleError {
    message: String,
}

impl LifecycleError {
    /// Creates a lifecycle error with a safe diagnostic message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ironic::LifecycleError;
    ///
    /// let err = LifecycleError::new("timeout");
    /// assert_eq!(format!("{err:?}"), "LifecycleError { message: \"timeout\" }");
    /// ```
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// ── Existing hooks ──────────────────────────────────────────────────

/// Runs after a provider's module and dependencies are available.
pub trait OnModuleInit: Send + Sync + 'static {
    /// Initializes the provider.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleError`] if initialization fails. The error message
    /// should be a safe diagnostic string that does not leak secrets.
    fn on_module_init(&self) -> LifecycleFuture<'_>;
}

/// Runs after every module initialization callback has succeeded.
pub trait OnApplicationBootstrap: Send + Sync + 'static {
    /// Completes application-level startup work.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleError`] if bootstrap fails. The application will
    /// abort startup and report the error.
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;
}

/// Runs during cleanup in reverse successful-initialization order.
pub trait OnModuleDestroy: Send + Sync + 'static {
    /// Releases module-owned resources.
    ///
    /// # Errors
    ///
    /// Errors are logged but do not prevent other modules from being destroyed.
    fn on_module_destroy(&self) -> LifecycleFuture<'_>;
}

/// Runs after serving stops and before module destruction.
pub trait OnApplicationShutdown: Send + Sync + 'static {
    /// Handles the shutdown signal.
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}

// ── New hooks ───────────────────────────────────────────────────────

/// Runs during module graph compilation, before any providers are built.
///
/// Use for: dynamic route registration, conditional provider setup,
/// validating module configuration.
pub trait OnModuleConfigure: Send + Sync + 'static {
    /// Receives the module's diagnostic name.
    fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_>;
}

/// Runs after the HTTP server binds to a port and is ready to accept connections.
///
/// Use for: self-health checks, notifying orchestrators, logging the bound address.
pub trait OnServerReady: Send + Sync + 'static {
    /// Called when the server is ready.
    fn on_server_ready(&self) -> LifecycleFuture<'_>;
}

/// Runs when a request-scoped provider is first resolved within a request.
///
/// Use for: per-request setup like initializing auth context, allocating
/// temporary resources, or logging the request identifier.
pub trait OnRequestInit: Send + Sync + 'static {
    /// `request_id` is the framework-generated request identifier.
    fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_>;
}

/// Runs when the request scope ends and the provider is about to be dropped.
///
/// Use for: closing temporary connections, flushing per-request metrics,
/// releasing resources acquired in `OnRequestInit`.
pub trait OnRequestDestroy: Send + Sync + 'static {
    /// Called when the owning request scope is dropped.
    fn on_request_destroy(&self) -> LifecycleFuture<'_>;
}

/// Called on every unhandled error before exception filters run.
///
/// Use for: centralized error logging, Sentry/DataDog reporting, alerting
/// on specific error codes across the entire application.
pub trait OnError: Send + Sync + 'static {
    /// `error_code` is the machine-readable code (e.g. `"POST_NOT_FOUND"`).
    /// `error_message` is the human-readable message.
    fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_>;
}

/// Called when any `Guard` returns `GuardDecision::Deny`.
///
/// Use for: centralized auth failure logging, brute-force detection,
/// rate-limit counters per guard type.
pub trait OnGuardDenied: Send + Sync + 'static {
    /// `guard_name` is the display name of the guard that denied the request.
    fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_>;
}

/// Runs immediately after a shutdown signal is received, BEFORE the server
/// stops accepting new connections.
///
/// Use for: draining in-flight connections, rejecting new requests gracefully,
/// signalling load balancers to stop routing traffic.
pub trait BeforeShutdown: Send + Sync + 'static {
    /// Receives the shutdown signal.
    fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}

/// Runs after ALL `OnModuleDestroy` callbacks have completed.
///
/// Use for: final metrics flush, last-chance cleanup, logging shutdown duration.
pub trait AfterShutdown: Send + Sync + 'static {
    /// Called after every module has been destroyed.
    fn after_shutdown(&self) -> LifecycleFuture<'_>;
}

/// Runs when a module is dynamically loaded after bootstrap.
pub trait OnModuleLoad: Send + Sync + 'static {
    /// `module_name` identifies the loaded module for diagnostics.
    fn on_module_load(&self, module_name: &str) -> LifecycleFuture<'_>;
}

/// Runs when a module is dynamically unloaded at runtime.
pub trait OnModuleUnload: Send + Sync + 'static {
    /// `module_name` identifies the unloaded module for diagnostics.
    fn on_module_unload(&self, module_name: &str) -> LifecycleFuture<'_>;
}

/// Runs after the DI container is built but before any lifecycle hooks fire.
///
/// Use for: connecting to databases, running migrations, or any async
/// initialization that requires the container to be available but must
/// complete before providers are constructed.
///
/// Unlike `OnModuleInit` (which runs on a per-provider basis), this trait
/// runs once per-module and receives access to the full container so it can
/// resolve other providers during initialization.
///
/// # Example
///
/// ```ignore
/// use ironic::AsyncModuleInit;
///
/// struct DatabaseModule;
///
/// impl AsyncModuleInit for DatabaseModule {
///     async fn async_init(&self, container: &ironic::Container) -> Result<(), ironic::AppError> {
///         let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;
///         // Register the pool as an eager provider, run migrations, etc.
///         Ok(())
///     }
/// }
/// ```
pub trait AsyncModuleInit: Send + Sync + 'static {
    /// Performs async initialization with access to the DI container.
    ///
    /// The container is fully built at this point and providers may be
    /// resolved. Use this for database connections, migrations, or any
    /// async startup that must complete before the HTTP server starts.
    ///
    /// # Errors
    ///
    /// Return a `LifecycleError` with a safe message. Do not leak credentials.
    fn async_init<'a>(&'a self, container: &'a ironic_di::Container) -> LifecycleFuture<'a>;
}

// ── LifecycleDefinition ─────────────────────────────────────────────

/// Type-erased lifecycle callbacks registered for one provider.
#[derive(Clone)]
pub struct LifecycleDefinition {
    key: ProviderKey,
    pub(crate) module_init: Option<InitCallback>,
    pub(crate) application_bootstrap: Option<InitCallback>,
    pub(crate) module_destroy: Option<InitCallback>,
    pub(crate) application_shutdown: Option<ShutdownCallback>,
    pub(crate) module_configure: Option<ConfigureCallback>,
    pub(crate) server_ready: Option<ServerReadyCallback>,
    pub(crate) request_init: Option<RequestInitCallback>,
    pub(crate) request_destroy: Option<RequestDestroyCallback>,
    pub(crate) on_error: Option<ErrorCallback>,
    pub(crate) guard_denied: Option<GuardDeniedCallback>,
    pub(crate) before_shutdown: Option<BeforeShutdownCallback>,
    pub(crate) after_shutdown: Option<AfterShutdownCallback>,
    pub(crate) module_load: Option<ModuleLoadCallback>,
    pub(crate) module_unload: Option<ModuleUnloadCallback>,
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
                module_configure: None,
                server_ready: None,
                request_init: None,
                request_destroy: None,
                on_error: None,
                guard_denied: None,
                before_shutdown: None,
                after_shutdown: None,
                module_load: None,
                module_unload: None,
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
            .field("module_configure", &self.module_configure.is_some())
            .field("server_ready", &self.server_ready.is_some())
            .field("request_init", &self.request_init.is_some())
            .field("request_destroy", &self.request_destroy.is_some())
            .field("on_error", &self.on_error.is_some())
            .field("guard_denied", &self.guard_denied.is_some())
            .field("before_shutdown", &self.before_shutdown.is_some())
            .field("after_shutdown", &self.after_shutdown.is_some())
            .field("module_load", &self.module_load.is_some())
            .field("module_unload", &self.module_unload.is_some())
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

    // ── New builder methods ─────────────────────────────────────────

    /// Registers [`OnModuleConfigure`].
    #[must_use]
    pub fn module_configure(mut self) -> Self
    where
        T: OnModuleConfigure,
    {
        self.definition.module_configure = Some(Arc::new(|value, name| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_module_configure(&name).await
            })
        }));
        self
    }

    /// Registers [`OnServerReady`].
    #[must_use]
    pub fn server_ready(mut self) -> Self
    where
        T: OnServerReady,
    {
        self.definition.server_ready = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_server_ready().await
            })
        }));
        self
    }

    /// Registers [`OnRequestInit`].
    #[must_use]
    pub fn request_init(mut self) -> Self
    where
        T: OnRequestInit,
    {
        self.definition.request_init = Some(Arc::new(|value, request_id| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_request_init(&request_id).await
            })
        }));
        self
    }

    /// Registers [`OnRequestDestroy`].
    #[must_use]
    pub fn request_destroy(mut self) -> Self
    where
        T: OnRequestDestroy,
    {
        self.definition.request_destroy = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_request_destroy().await
            })
        }));
        self
    }

    /// Registers [`OnError`].
    #[must_use]
    pub fn on_error(mut self) -> Self
    where
        T: OnError,
    {
        self.definition.on_error = Some(Arc::new(|value, code, msg| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_error(&code, &msg).await
            })
        }));
        self
    }

    /// Registers [`OnGuardDenied`].
    #[must_use]
    pub fn guard_denied(mut self) -> Self
    where
        T: OnGuardDenied,
    {
        self.definition.guard_denied = Some(Arc::new(|value, name| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_guard_denied(&name).await
            })
        }));
        self
    }

    /// Registers [`BeforeShutdown`].
    #[must_use]
    pub fn before_shutdown(mut self) -> Self
    where
        T: BeforeShutdown,
    {
        self.definition.before_shutdown = Some(Arc::new(|value, signal| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.before_shutdown(signal).await
            })
        }));
        self
    }

    /// Registers [`AfterShutdown`].
    #[must_use]
    pub fn after_shutdown(mut self) -> Self
    where
        T: AfterShutdown,
    {
        self.definition.after_shutdown = Some(Arc::new(|value| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.after_shutdown().await
            })
        }));
        self
    }

    /// Registers [`OnModuleLoad`].
    #[must_use]
    pub fn module_load(mut self) -> Self
    where
        T: OnModuleLoad,
    {
        self.definition.module_load = Some(Arc::new(|value, name| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_module_load(&name).await
            })
        }));
        self
    }

    /// Registers [`OnModuleUnload`].
    #[must_use]
    pub fn module_unload(mut self) -> Self
    where
        T: OnModuleUnload,
    {
        self.definition.module_unload = Some(Arc::new(|value, name| {
            Box::pin(async move {
                let provider = downcast::<T>(value)?;
                provider.on_module_unload(&name).await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_error_new_and_display() {
        let err = LifecycleError::new("something went wrong");
        assert_eq!(err.message, "something went wrong");
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn lifecycle_error_debug() {
        let err = LifecycleError::new("oh no");
        let debug = format!("{err:?}");
        assert!(debug.contains("oh no"));
    }

    #[test]
    fn lifecycle_error_partial_eq() {
        let a = LifecycleError::new("same");
        let b = LifecycleError::new("same");
        let c = LifecycleError::new("different");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn lifecycle_error_clone() {
        let a = LifecycleError::new("clone me");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn lifecycle_definition_builder_defaults() {
        struct MyService;
        let def = LifecycleDefinition::builder::<MyService>().build();
        assert_eq!(def.key(), ProviderKey::of::<MyService>());
        assert!(def.module_init.is_none());
        assert!(def.application_bootstrap.is_none());
        assert!(def.module_destroy.is_none());
        assert!(def.application_shutdown.is_none());
        assert!(def.module_configure.is_none());
        assert!(def.server_ready.is_none());
        assert!(def.request_init.is_none());
        assert!(def.request_destroy.is_none());
        assert!(def.on_error.is_none());
        assert!(def.guard_denied.is_none());
        assert!(def.before_shutdown.is_none());
        assert!(def.after_shutdown.is_none());
        assert!(def.module_load.is_none());
        assert!(def.module_unload.is_none());
    }

    #[test]
    fn lifecycle_definition_debug_output() {
        struct MyService;
        let def = LifecycleDefinition::builder::<MyService>().build();
        let debug = format!("{def:?}");
        assert!(debug.contains("LifecycleDefinition"));
        assert!(debug.contains("module_init: false"));
    }
}
