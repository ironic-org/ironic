use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use ironic_di::{Container, ProviderDefinition, ProviderKey, ProviderValue, ResolveError};
use ironic_http::{Middleware, RequestLogging};
use ironic_platform::{HttpPlatformAdapter, HttpPlatformApplication, Shutdown, ShutdownSignal};

use crate::{
    CompiledApplicationGraph, HttpApplicationBuildError, LifecycleDefinition, LifecycleError,
    ModuleDefinition, ModuleError, ModuleRef, build_http_application_with_extra_providers,
    compile_module_graph,
};

/// Marker used until an application builder receives a platform adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct MissingPlatform;

/// Application build, serving, or lifecycle failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ApplicationError {
    /// No root module was configured.
    #[error("RF_APP_MISSING_ROOT_MODULE: configure a root module before building")]
    MissingRootModule,
    /// Asynchronous root module configuration failed.
    #[error("IRONIC_APP_MODULE_CONFIGURATION: {message}")]
    ModuleConfiguration {
        /// A safe configuration failure message.
        message: String,
    },
    /// The root module graph is invalid.
    #[error(transparent)]
    Module(Box<ModuleError>),
    /// HTTP runtime compilation failed.
    #[error(transparent)]
    Http(#[from] HttpApplicationBuildError),
    /// An eager provider could not be constructed.
    #[error("RF_APP_EAGER_PROVIDER_FAILED: `{provider}`: {message}")]
    EagerProvider {
        /// The provider that failed.
        provider: ProviderKey,
        /// A safe resolution message.
        message: String,
    },
    /// A lifecycle callback failed.
    #[error("RF_APP_LIFECYCLE_FAILED: `{provider}` during {stage}: {message}")]
    Lifecycle {
        /// The lifecycle provider.
        provider: ProviderKey,
        /// The lifecycle stage.
        stage: &'static str,
        /// A safe callback message.
        message: String,
    },
    /// Native platform construction or serving failed.
    #[error("RF_APP_PLATFORM_FAILED: {message}")]
    Platform {
        /// A safe platform message.
        message: String,
    },
    /// A listening address could not be parsed.
    #[error("RF_APP_INVALID_ADDRESS: `{address}` is not a socket address")]
    InvalidAddress {
        /// The invalid address.
        address: String,
    },
}

impl From<ModuleError> for ApplicationError {
    fn from(err: ModuleError) -> Self {
        Self::Module(Box::new(err))
    }
}

/// Starts application construction from a root module and platform adapter.
pub struct ApplicationBuilder<A = MissingPlatform> {
    root: Option<RootModule>,
    overrides: Vec<ironic_di::ProviderDefinition>,
    middlewares: Vec<Arc<dyn Middleware>>,
    adapter: A,
    disable_request_logging: bool,
}

type ModuleConfigurationFuture =
    Pin<Box<dyn Future<Output = Result<ModuleDefinition, ModuleConfigurationError>> + Send>>;

enum RootModule {
    Ready(ModuleDefinition),
    Deferred(ModuleConfigurationFuture),
}

/// A safe error returned by asynchronous module configuration.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct ModuleConfigurationError {
    message: String,
}

impl ModuleConfigurationError {
    /// Creates a safe configuration error. Do not include credentials in `message`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for ApplicationBuilder<MissingPlatform> {
    fn default() -> Self {
        Self {
            root: None,
            overrides: Vec::new(),
            middlewares: Vec::new(),
            adapter: MissingPlatform,
            disable_request_logging: false,
        }
    }
}

impl<A> ApplicationBuilder<A> {
    /// Sets the explicit root module definition.
    #[must_use]
    pub fn module(mut self, module: ModuleDefinition) -> Self {
        self.root = Some(RootModule::Ready(module));
        self
    }

    /// Defers root module construction until asynchronous application build.
    ///
    /// This supports configuration loaded from secret managers, service discovery, or other
    /// asynchronous sources without introducing a second module compiler.
    #[must_use]
    pub fn module_async<F>(mut self, module: F) -> Self
    where
        F: Future<Output = Result<ModuleDefinition, ModuleConfigurationError>> + Send + 'static,
    {
        self.root = Some(RootModule::Deferred(Box::pin(module)));
        self
    }

    /// Replaces one provider registration in this application build.
    ///
    /// The override must use the same concrete provider key as an existing module registration.
    #[must_use]
    pub fn override_provider(mut self, provider: ironic_di::ProviderDefinition) -> Self {
        self.overrides.push(provider);
        self
    }

    /// Registers global middleware that runs before all controllers and routes.
    ///
    /// Middleware is registered in the order it is added and wraps the entire
    /// request pipeline (global → controller → route → handler).
    #[must_use]
    pub fn middleware(mut self, middleware: impl Middleware) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Disables the default [`RequestLogging`] middleware.
    ///
    /// By default, every request is logged as a structured tracing event. Call
    /// this method to opt out.
    #[must_use]
    pub fn without_request_logging(mut self) -> Self {
        self.disable_request_logging = true;
        self
    }

    /// Selects a concrete HTTP platform adapter.
    #[must_use]
    pub fn platform<B>(self, adapter: B) -> ApplicationBuilder<B> {
        ApplicationBuilder {
            root: self.root,
            overrides: self.overrides,
            middlewares: self.middlewares,
            adapter,
            disable_request_logging: self.disable_request_logging,
        }
    }
}

impl<A> ApplicationBuilder<A>
where
    A: HttpPlatformAdapter,
{
    /// Compiles, initializes, and builds the application.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when graph compilation, provider initialization, lifecycle
    /// callbacks, or platform construction fails. Successfully initialized lifecycle providers are
    /// destroyed before an error is returned.
    pub async fn build(self) -> Result<Application<A::Application>, ApplicationError> {
        let root = match self.root.ok_or(ApplicationError::MissingRootModule)? {
            RootModule::Ready(module) => module,
            RootModule::Deferred(module) => {
                module
                    .await
                    .map_err(|error| ApplicationError::ModuleConfiguration {
                        message: error.to_string(),
                    })?
            }
        };
        let module_ref = std::sync::Arc::new(ModuleRef::new());
        let module_ref_provider = ProviderDefinition::value(module_ref.clone());
        let graph = compile_module_graph(root)?;
        let http = build_http_application_with_extra_providers(
            &graph,
            [module_ref_provider],
            self.overrides,
        )?
        .extend_middleware(self.middlewares);
        let http = if self.disable_request_logging {
            http
        } else {
            http.middleware(RequestLogging::new())
        };
        let container = http.container().clone();
        module_ref.set_container(container.clone());

        configure_modules(&graph, &container).await?;
        run_async_module_init(&graph, &container).await?;
        initialize_eager_providers(&graph, &container).await?;
        let mut initialized = Vec::new();
        if let Err(error) = initialize_lifecycle(&graph, &container, &mut initialized).await {
            let _ = destroy_modules(&initialized).await;
            return Err(error);
        }
        if let Err(error) = bootstrap_application(&initialized).await {
            let _ = destroy_modules(&initialized).await;
            return Err(error);
        }
        if let Err(error) = server_ready(&initialized).await {
            let _ = destroy_modules(&initialized).await;
            return Err(error);
        }

        let platform = match self.adapter.build(Arc::new(http)) {
            Ok(platform) => platform,
            Err(error) => {
                let _ = destroy_modules(&initialized).await;
                return Err(ApplicationError::Platform {
                    message: error.to_string(),
                });
            }
        };

        Ok(Application {
            graph,
            container,
            platform,
            initialized,
        })
    }
}

/// A compiled and initialized application ready to listen.
pub struct Application<P = ()> {
    graph: CompiledApplicationGraph,
    container: Container,
    platform: P,
    initialized: Vec<InitializedLifecycle>,
}

impl Application<()> {
    /// Creates an empty application builder.
    #[must_use]
    pub fn builder() -> ApplicationBuilder<MissingPlatform> {
        ApplicationBuilder::default()
    }
}

impl<P> Application<P> {
    /// Returns the validated application graph.
    #[must_use]
    pub const fn graph(&self) -> &CompiledApplicationGraph {
        &self.graph
    }

    /// Returns the initialized application container.
    #[must_use]
    pub const fn container(&self) -> &Container {
        &self.container
    }

    /// Returns the built platform application.
    #[must_use]
    pub const fn platform(&self) -> &P {
        &self.platform
    }

    /// Runs application shutdown and module-destruction hooks without starting a listener.
    ///
    /// This is useful for in-process applications and tests that still require deterministic
    /// lifecycle cleanup.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when a shutdown or destruction callback fails.
    pub async fn shutdown(self, signal: ShutdownSignal) -> Result<(), ApplicationError> {
        shutdown_application(&self.initialized, signal).await
    }
}

impl<P> Application<P>
where
    P: HttpPlatformApplication,
{
    /// Listens until the process receives Ctrl-C, then runs shutdown hooks.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when the address is invalid, serving fails, or cleanup fails.
    pub async fn listen(self, address: &str) -> Result<(), ApplicationError> {
        let parsed =
            address
                .parse::<SocketAddr>()
                .map_err(|_| ApplicationError::InvalidAddress {
                    address: address.to_owned(),
                })?;
        self.listen_with_shutdown(parsed, async {
            let _ = tokio::signal::ctrl_c().await;
            ShutdownSignal::Interrupt
        })
        .await
    }

    /// Listens until a caller-provided shutdown future completes.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when serving or cleanup fails.
    pub async fn listen_with_shutdown<F>(
        self,
        address: SocketAddr,
        shutdown: F,
    ) -> Result<(), ApplicationError>
    where
        F: Future<Output = ShutdownSignal> + Send + 'static,
    {
        let Application {
            platform,
            initialized,
            ..
        } = self;

        // Wrap the shutdown future so BeforeShutdown callbacks run
        // BEFORE the server stops accepting connections.
        let init_for_shutdown = initialized.clone();
        let wrapped = async move {
            let signal = shutdown.await;
            for lifecycle in &init_for_shutdown {
                if let Some(callback) = &lifecycle.definition.before_shutdown {
                    let _ = callback(Arc::clone(&lifecycle.provider), signal).await;
                }
            }
            signal
        };

        let serving = platform.listen(address, Shutdown::new(wrapped)).await;
        let signal = match &serving {
            Ok(signal) => *signal,
            Err(_) => ShutdownSignal::Custom("platform-error"),
        };

        let cleanup = shutdown_application(&initialized, signal).await;
        if let Err(error) = serving {
            return Err(ApplicationError::Platform {
                message: error.to_string(),
            });
        }
        cleanup
    }
}

#[derive(Clone)]
struct InitializedLifecycle {
    definition: LifecycleDefinition,
    provider: ProviderValue,
}

async fn run_async_module_init(
    graph: &CompiledApplicationGraph,
    container: &Container,
) -> Result<(), ApplicationError> {
    let container = std::sync::Arc::new(container.clone());
    let dummy_key = ironic_di::ProviderKey::of::<()>();
    let dummy_provider: ironic_di::ProviderValue = std::sync::Arc::new(());
    for module_id in graph.initialization_order() {
        let module = graph
            .module(*module_id)
            .expect("initialization order references a compiled module");
        for callback in module.async_init_callbacks() {
            callback(
                std::sync::Arc::clone(&dummy_provider),
                std::sync::Arc::clone(&container),
            )
            .await
            .map_err(|error| ApplicationError::Lifecycle {
                provider: dummy_key,
                stage: "async_module_init",
                message: error.to_string(),
            })?;
        }
    }
    Ok(())
}

async fn initialize_eager_providers(
    graph: &CompiledApplicationGraph,
    container: &Container,
) -> Result<(), ApplicationError> {
    for module_id in graph.initialization_order() {
        let module = graph
            .module(*module_id)
            .expect("initialization order references a compiled module");
        for provider in module
            .providers()
            .iter()
            .filter(|provider| provider.is_eager())
        {
            container
                .resolve_key(provider.key())
                .await
                .map_err(|error| ApplicationError::EagerProvider {
                    provider: provider.key(),
                    message: resolution_message(&error),
                })?;
        }
    }
    Ok(())
}

async fn initialize_lifecycle(
    graph: &CompiledApplicationGraph,
    container: &Container,
    initialized: &mut Vec<InitializedLifecycle>,
) -> Result<(), ApplicationError> {
    for module_id in graph.initialization_order() {
        let module = graph
            .module(*module_id)
            .expect("initialization order references a compiled module");
        for definition in module.lifecycle() {
            let provider = container
                .resolve_key(definition.key())
                .await
                .map_err(|error| ApplicationError::EagerProvider {
                    provider: definition.key(),
                    message: resolution_message(&error),
                })?;
            initialized.push(InitializedLifecycle {
                definition: definition.clone(),
                provider: Arc::clone(&provider),
            });
            if let Some(callback) = &definition.module_init {
                callback(provider).await.map_err(|error| {
                    lifecycle_error(definition.key(), "module initialization", &error)
                })?;
            }
        }
    }
    Ok(())
}

async fn bootstrap_application(
    initialized: &[InitializedLifecycle],
) -> Result<(), ApplicationError> {
    for lifecycle in initialized {
        if let Some(callback) = &lifecycle.definition.application_bootstrap {
            callback(Arc::clone(&lifecycle.provider))
                .await
                .map_err(|error| {
                    lifecycle_error(lifecycle.definition.key(), "application bootstrap", &error)
                })?;
        }
    }
    Ok(())
}

async fn shutdown_application(
    initialized: &[InitializedLifecycle],
    signal: ShutdownSignal,
) -> Result<(), ApplicationError> {
    // BeforeShutdown already ran in listen_with_shutdown before server stopped.
    // Continue with application shutdown and module destruction.

    let mut first_error = None;
    for lifecycle in initialized.iter().rev() {
        if let Some(callback) = &lifecycle.definition.application_shutdown
            && let Err(error) = callback(Arc::clone(&lifecycle.provider), signal).await
        {
            first_error.get_or_insert_with(|| {
                lifecycle_error(lifecycle.definition.key(), "application shutdown", &error)
            });
        }
    }
    if let Err(error) = destroy_modules(initialized).await {
        first_error.get_or_insert(error);
    }

    // AfterShutdown: final cleanup after all destroy callbacks
    for lifecycle in initialized.iter().rev() {
        if let Some(callback) = &lifecycle.definition.after_shutdown {
            let _ = callback(Arc::clone(&lifecycle.provider)).await;
        }
    }

    first_error.map_or(Ok(()), Err)
}

async fn destroy_modules(initialized: &[InitializedLifecycle]) -> Result<(), ApplicationError> {
    let mut first_error = None;
    for lifecycle in initialized.iter().rev() {
        if let Some(callback) = &lifecycle.definition.module_destroy
            && let Err(error) = callback(Arc::clone(&lifecycle.provider)).await
        {
            first_error.get_or_insert_with(|| {
                lifecycle_error(lifecycle.definition.key(), "module destruction", &error)
            });
        }
    }
    first_error.map_or(Ok(()), Err)
}

fn lifecycle_error(
    provider: ProviderKey,
    stage: &'static str,
    error: &LifecycleError,
) -> ApplicationError {
    ApplicationError::Lifecycle {
        provider,
        stage,
        message: error.to_string(),
    }
}

fn resolution_message(error: &ResolveError) -> String {
    error.to_string()
}

async fn configure_modules(
    graph: &CompiledApplicationGraph,
    container: &Container,
) -> Result<(), ApplicationError> {
    for module_id in graph.initialization_order() {
        let module = graph
            .module(*module_id)
            .expect("initialization order references a compiled module");
        let module_name = format!("{:?}", module.id());
        for definition in module.lifecycle() {
            if let Some(callback) = &definition.module_configure {
                let provider = container
                    .resolve_key(definition.key())
                    .await
                    .map_err(|error| ApplicationError::EagerProvider {
                        provider: definition.key(),
                        message: resolution_message(&error),
                    })?;
                callback(provider, module_name.clone())
                    .await
                    .map_err(|error| {
                        lifecycle_error(definition.key(), "module configuration", &error)
                    })?;
            }
        }
    }
    Ok(())
}

async fn server_ready(initialized: &[InitializedLifecycle]) -> Result<(), ApplicationError> {
    for lifecycle in initialized {
        if let Some(callback) = &lifecycle.definition.server_ready {
            callback(Arc::clone(&lifecycle.provider))
                .await
                .map_err(|error| {
                    lifecycle_error(lifecycle.definition.key(), "server ready", &error)
                })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use ironic_di::{ProviderDefinition, Scope};
    use ironic_http::CompiledHttpApplication;
    use ironic_platform::{PlatformFuture, Shutdown};

    use super::*;
    use crate::{
        LifecycleDefinition, LifecycleFuture, Module, ModuleId, OnApplicationBootstrap,
        OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
    };

    type Events = Arc<Mutex<Vec<&'static str>>>;

    fn push(events: &Events, event: &'static str) {
        events.lock().unwrap().push(event);
    }

    struct FirstLifecycle {
        events: Events,
    }

    struct SecondLifecycle {
        events: Events,
        fail_init: bool,
    }

    macro_rules! lifecycle_impls {
        ($type:ty, $init:literal, $bootstrap:literal, $shutdown:literal, $destroy:literal) => {
            impl OnApplicationBootstrap for $type {
                fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
                    Box::pin(async move {
                        push(&self.events, $bootstrap);
                        Ok(())
                    })
                }
            }

            impl OnApplicationShutdown for $type {
                fn on_application_shutdown(&self, _signal: ShutdownSignal) -> LifecycleFuture<'_> {
                    Box::pin(async move {
                        push(&self.events, $shutdown);
                        Ok(())
                    })
                }
            }

            impl OnModuleDestroy for $type {
                fn on_module_destroy(&self) -> LifecycleFuture<'_> {
                    Box::pin(async move {
                        push(&self.events, $destroy);
                        Ok(())
                    })
                }
            }
        };
    }

    lifecycle_impls!(
        FirstLifecycle,
        "first-init",
        "first-bootstrap",
        "first-shutdown",
        "first-destroy"
    );
    lifecycle_impls!(
        SecondLifecycle,
        "second-init",
        "second-bootstrap",
        "second-shutdown",
        "second-destroy"
    );

    impl OnModuleInit for FirstLifecycle {
        fn on_module_init(&self) -> LifecycleFuture<'_> {
            Box::pin(async move {
                push(&self.events, "first-init");
                Ok(())
            })
        }
    }

    impl OnModuleInit for SecondLifecycle {
        fn on_module_init(&self) -> LifecycleFuture<'_> {
            Box::pin(async move {
                push(&self.events, "second-init");
                if self.fail_init {
                    Err(LifecycleError::new("second initialization failed"))
                } else {
                    Ok(())
                }
            })
        }
    }

    struct TestModule;
    impl Module for TestModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>().build()
        }
    }

    fn test_module(events: &Events, fail_second_init: bool) -> ModuleDefinition {
        let first_events = Arc::clone(events);
        let second_events = Arc::clone(events);
        ModuleDefinition::builder::<TestModule>()
            .provider(
                ProviderDefinition::constructor(Scope::Singleton, Vec::new(), move |_resolver| {
                    push(&first_events, "first-construct");
                    Ok(FirstLifecycle {
                        events: Arc::clone(&first_events),
                    })
                })
                .eager(),
            )
            .provider(
                ProviderDefinition::constructor(Scope::Singleton, Vec::new(), move |_resolver| {
                    push(&second_events, "second-construct");
                    Ok(SecondLifecycle {
                        events: Arc::clone(&second_events),
                        fail_init: fail_second_init,
                    })
                })
                .eager(),
            )
            .lifecycle(
                LifecycleDefinition::builder::<FirstLifecycle>()
                    .module_init()
                    .application_bootstrap()
                    .application_shutdown()
                    .module_destroy()
                    .build(),
            )
            .lifecycle(
                LifecycleDefinition::builder::<SecondLifecycle>()
                    .module_init()
                    .application_bootstrap()
                    .application_shutdown()
                    .module_destroy()
                    .build(),
            )
            .build()
    }

    #[derive(Clone)]
    struct FakeAdapter {
        events: Events,
        fail_build: bool,
    }

    struct FakeApplication {
        events: Events,
    }

    #[derive(Clone, Debug, thiserror::Error)]
    #[error("fake platform error")]
    struct FakePlatformError;

    impl HttpPlatformAdapter for FakeAdapter {
        type Application = FakeApplication;
        type Error = FakePlatformError;

        fn build(
            self,
            _application: Arc<CompiledHttpApplication>,
        ) -> Result<Self::Application, Self::Error> {
            push(&self.events, "platform-build");
            if self.fail_build {
                Err(FakePlatformError)
            } else {
                Ok(FakeApplication {
                    events: self.events,
                })
            }
        }
    }

    impl HttpPlatformApplication for FakeApplication {
        type Error = FakePlatformError;

        fn listen(
            self,
            _address: SocketAddr,
            shutdown: Shutdown,
        ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
            Box::pin(async move {
                push(&self.events, "listen");
                let signal = shutdown.wait().await;
                push(&self.events, "serve-stop");
                Ok(signal)
            })
        }
    }

    #[tokio::test]
    async fn runs_complete_lifecycle_in_deterministic_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let application = Application::builder()
            .module(test_module(&events, false))
            .platform(FakeAdapter {
                events: Arc::clone(&events),
                fail_build: false,
            })
            .build()
            .await
            .unwrap();

        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "first-construct",
                "second-construct",
                "first-init",
                "second-init",
                "first-bootstrap",
                "second-bootstrap",
                "platform-build",
            ]
        );

        application
            .listen_with_shutdown(
                "127.0.0.1:0".parse().unwrap(),
                std::future::ready(ShutdownSignal::Custom("test")),
            )
            .await
            .unwrap();
        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "first-construct",
                "second-construct",
                "first-init",
                "second-init",
                "first-bootstrap",
                "second-bootstrap",
                "platform-build",
                "listen",
                "serve-stop",
                "second-shutdown",
                "first-shutdown",
                "second-destroy",
                "first-destroy",
            ]
        );
    }

    #[tokio::test]
    async fn cleans_up_partially_initialized_applications() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let result = Application::builder()
            .module(test_module(&events, true))
            .platform(FakeAdapter {
                events: Arc::clone(&events),
                fail_build: false,
            })
            .build()
            .await;

        assert!(matches!(result, Err(ApplicationError::Lifecycle { .. })));
        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "first-construct",
                "second-construct",
                "first-init",
                "second-init",
                "second-destroy",
                "first-destroy",
            ]
        );
    }

    #[tokio::test]
    async fn cleans_up_when_platform_build_fails() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let result = Application::builder()
            .module(test_module(&events, false))
            .platform(FakeAdapter {
                events: Arc::clone(&events),
                fail_build: true,
            })
            .build()
            .await;

        assert!(matches!(result, Err(ApplicationError::Platform { .. })));
        assert_eq!(
            events.lock().unwrap().as_slice().last_chunk::<3>(),
            Some(&["platform-build", "second-destroy", "first-destroy"])
        );
    }

    #[tokio::test]
    async fn rejects_builders_without_a_root_module() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let result = Application::builder()
            .platform(FakeAdapter {
                events,
                fail_build: false,
            })
            .build()
            .await;
        assert!(matches!(result, Err(ApplicationError::MissingRootModule)));
    }

    #[tokio::test]
    async fn asynchronously_configures_the_root_module() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let application = Application::builder()
            .module_async(async {
                tokio::task::yield_now().await;
                Ok(TestModule::definition())
            })
            .platform(FakeAdapter {
                events: Arc::clone(&events),
                fail_build: false,
            })
            .build()
            .await
            .unwrap();

        assert_eq!(application.graph().root(), ModuleId::of::<TestModule>());
        assert_eq!(events.lock().unwrap().as_slice(), ["platform-build"]);
    }

    #[tokio::test]
    async fn reports_async_module_configuration_failures() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let result = Application::builder()
            .module_async(async {
                Err(ModuleConfigurationError::new(
                    "remote configuration is unavailable",
                ))
            })
            .platform(FakeAdapter {
                events,
                fail_build: false,
            })
            .build()
            .await;

        assert!(matches!(
            result,
            Err(ApplicationError::ModuleConfiguration { .. })
        ));
    }
}
