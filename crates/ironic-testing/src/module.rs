use std::{future::Future, sync::Arc};

use ironic_core::{
    CompiledApplicationGraph, Module, build_http_application_with_overrides, compile_module_graph,
};
use ironic_di::{Container, Dependency, ProviderDefinition, ResolveError, Scope};

use crate::TestBuildError;

/// Entry point for compiling a module with test-local provider overrides.
#[derive(Clone, Copy, Debug, Default)]
pub struct TestModule;

impl TestModule {
    /// Starts an isolated compiler for module `M`.
    #[must_use]
    pub fn builder<M: Module>() -> TestModuleBuilder {
        TestModuleBuilder {
            root: M::definition(),
            overrides: Vec::new(),
        }
    }
}

/// Builds an isolated module container.
pub struct TestModuleBuilder {
    root: ironic_core::ModuleDefinition,
    overrides: Vec<ProviderDefinition>,
}

impl TestModuleBuilder {
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

    /// Replaces provider `T` with an asynchronous factory.
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

    /// Compiles the module graph, applies overrides, and initializes eager providers.
    ///
    /// # Errors
    ///
    /// Returns [`TestBuildError`] for invalid graphs, overrides, routes, or eager providers.
    pub async fn compile(self) -> Result<CompiledTestModule, TestBuildError> {
        let graph = compile_module_graph(self.root)?;
        let application = build_http_application_with_overrides(&graph, self.overrides)?;
        let container = application.container().clone();
        for module_id in graph.initialization_order() {
            let Some(module) = graph.module(*module_id) else {
                continue;
            };
            for provider in module
                .providers()
                .iter()
                .filter(|provider| provider.is_eager())
            {
                container.resolve_key(provider.key()).await?;
            }
        }
        Ok(CompiledTestModule { graph, container })
    }
}

/// A validated module graph and its isolated dependency container.
pub struct CompiledTestModule {
    graph: CompiledApplicationGraph,
    container: Container,
}

impl CompiledTestModule {
    /// Returns the validated module graph.
    #[must_use]
    pub const fn graph(&self) -> &CompiledApplicationGraph {
        &self.graph
    }

    /// Resolves a provider from the isolated container.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when the provider is missing or construction fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        self.container.resolve::<T>().await
    }
}
