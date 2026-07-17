use std::{convert::Infallible, future::Future, net::SocketAddr, sync::Arc};

use ironic_core::{Application, ApplicationError, Module};
use ironic_di::{Dependency, ProviderDefinition, ResolveError, Scope};
use ironic_http::{CompiledHttpApplication, HttpMethod};
use ironic_platform::{
    HttpPlatformAdapter, HttpPlatformApplication, PlatformFuture, Shutdown, ShutdownSignal,
};

use crate::TestRequestBuilder;

/// Builds an in-process HTTP application with test-local provider overrides.
pub struct TestApplicationBuilder {
    root: ironic_core::ModuleDefinition,
    overrides: Vec<ProviderDefinition>,
}

impl TestApplicationBuilder {
    /// Replaces a provider with a complete definition using the same concrete key.
    #[must_use]
    pub fn override_provider(mut self, provider: ProviderDefinition) -> Self {
        self.overrides.push(provider);
        self
    }

    /// Replaces provider `T` with a singleton test value.
    #[must_use]
    pub fn override_value<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        self.overrides.push(ProviderDefinition::value(value));
        self
    }

    /// Replaces provider `T` with an asynchronous test factory.
    #[must_use]
    pub fn override_factory<T, F, Fut>(
        mut self,
        scope: Scope,
        dependencies: Vec<Dependency>,
        factory: F,
    ) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(ironic_di::Resolver) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, ResolveError>> + Send + 'static,
    {
        self.overrides
            .push(ProviderDefinition::factory(scope, dependencies, factory));
        self
    }

    /// Compiles and initializes the in-process test application.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when the graph, overrides, eager providers, routes, or
    /// lifecycle hooks fail.
    pub async fn build(self) -> Result<TestApplication, ApplicationError> {
        let mut builder = Application::builder()
            .module(self.root)
            .platform(InProcessAdapter);
        for provider in self.overrides {
            builder = builder.override_provider(provider);
        }
        Ok(TestApplication {
            application: Some(builder.build().await?),
        })
    }
}

/// A complete application that dispatches requests without binding a network port.
pub struct TestApplication {
    application: Option<Application<InProcessApplication>>,
}

impl TestApplication {
    /// Builds a test application for root module `M`.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when application initialization fails.
    pub async fn new<M: Module>() -> Result<Self, ApplicationError> {
        Self::builder::<M>().build().await
    }

    /// Starts a configurable test application builder for root module `M`.
    #[must_use]
    pub fn builder<M: Module>() -> TestApplicationBuilder {
        TestApplicationBuilder {
            root: M::definition(),
            overrides: Vec::new(),
        }
    }

    /// Resolves an application provider.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when resolution fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        self.application().container().resolve::<T>().await
    }

    /// Starts a `GET` request.
    #[must_use]
    pub fn get(&self, path: impl Into<String>) -> TestRequestBuilder<'_> {
        self.request(HttpMethod::GET, path)
    }

    /// Starts a `POST` request.
    #[must_use]
    pub fn post(&self, path: impl Into<String>) -> TestRequestBuilder<'_> {
        self.request(HttpMethod::POST, path)
    }

    /// Starts a `PUT` request.
    #[must_use]
    pub fn put(&self, path: impl Into<String>) -> TestRequestBuilder<'_> {
        self.request(HttpMethod::PUT, path)
    }

    /// Starts a `PATCH` request.
    #[must_use]
    pub fn patch(&self, path: impl Into<String>) -> TestRequestBuilder<'_> {
        self.request(HttpMethod::PATCH, path)
    }

    /// Starts a `DELETE` request.
    #[must_use]
    pub fn delete(&self, path: impl Into<String>) -> TestRequestBuilder<'_> {
        self.request(HttpMethod::DELETE, path)
    }

    /// Starts a request with an arbitrary HTTP method.
    #[must_use]
    pub fn request(&self, method: HttpMethod, path: impl Into<String>) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self.application().platform().http(), method, path.into())
    }

    /// Runs shutdown and destruction hooks in deterministic reverse order.
    ///
    /// Call this at the end of every test that creates an application.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] when a cleanup hook fails.
    pub async fn shutdown(mut self) -> Result<(), ApplicationError> {
        let Some(application) = self.application.take() else {
            return Ok(());
        };
        application
            .shutdown(ShutdownSignal::Custom("test-complete"))
            .await
    }

    fn application(&self) -> &Application<InProcessApplication> {
        self.application
            .as_ref()
            .expect("test application has not been shut down")
    }
}

impl Drop for TestApplication {
    fn drop(&mut self) {
        let Some(application) = self.application.take() else {
            return;
        };
        let cleanup = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Ironic test cleanup runtime must initialize")
                .block_on(application.shutdown(ShutdownSignal::Custom("test-drop")))
        })
        .join();

        if std::thread::panicking() {
            return;
        }
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!("Ironic test application cleanup failed: {error}"),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("unknown panic payload");
                panic!("Ironic test application cleanup panicked: {message}");
            }
        }
    }
}

struct InProcessAdapter;

impl HttpPlatformAdapter for InProcessAdapter {
    type Application = InProcessApplication;
    type Error = Infallible;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error> {
        Ok(InProcessApplication { application })
    }
}

/// Platform state used only by the test application.
pub(crate) struct InProcessApplication {
    application: Arc<CompiledHttpApplication>,
}

impl InProcessApplication {
    pub(crate) fn http(&self) -> &CompiledHttpApplication {
        &self.application
    }
}

impl HttpPlatformApplication for InProcessApplication {
    type Error = Infallible;

    fn listen(
        self,
        _address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
        Box::pin(async move { Ok(shutdown.wait().await) })
    }
}
